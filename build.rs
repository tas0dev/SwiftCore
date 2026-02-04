use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let apps_dir = manifest_dir.join("src/apps");

    let initfs_dir_core = manifest_dir.join("src/initfs");
    let initfs_dir_legacy = manifest_dir.join("src/init/initfs");
    let initfs_dir = if initfs_dir_core.is_dir() {
        initfs_dir_core
    } else {
        initfs_dir_legacy
    };

    if !initfs_dir.is_dir() {
        panic!("initfs directory not found at {:?}", initfs_dir);
    }

    // appsディレクトリが存在する場合、アプリをビルド
    if apps_dir.is_dir() {
        build_apps(&apps_dir, &initfs_dir);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let image_path = out_dir.join("initfs.ext2");

    emit_rerun_if_changed(&initfs_dir);

    let status = Command::new("mke2fs")
        .args(["-t", "ext2", "-b", "4096", "-m", "0", "-L", "initfs", "-d"])
        .arg(&initfs_dir)
        .arg(&image_path)
        .arg("4096")
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            panic!("mke2fs failed while generating initfs.ext2");
        }
        Err(e) => {
            panic!("failed to execute mke2fs: {e}. Please install e2fsprogs (mke2fs).");
        }
    }
}

fn build_apps(apps_dir: &Path, initfs_dir: &Path) {
    println!("cargo:rerun-if-changed={}", apps_dir.display());

    let entries = match fs::read_dir(apps_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let app_name = path.file_name().unwrap().to_string_lossy();
        println!("cargo:warning=Building app: {}", app_name);

        emit_rerun_if_changed(&path);

        // cargoでアプリをビルド
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&path)
            .status();

        match status {
            Ok(s) if s.success() => {
                // ビルド成果物を探す
                let target_dir = path.join("target");
                if let Some(elf_path) = find_built_binary(&target_dir) {
                    // initfsにコピー
                    let dest = initfs_dir.join(&*app_name);
                    if let Err(e) = fs::copy(&elf_path, &dest) {
                        println!("cargo:warning=Failed to copy {} to initfs: {}", app_name, e);
                    } else {
                        println!("cargo:warning=Copied {} to initfs", app_name);
                    }
                }
            }
            Ok(_) => {
                println!("cargo:warning=Failed to build app: {}", app_name);
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute cargo for {}: {}", app_name, e);
            }
        }
    }
}

fn find_built_binary(target_dir: &Path) -> Option<PathBuf> {
    // x86_64-swiftcore/release/ を優先的に探す
    let custom_target = target_dir.join("x86_64-swiftcore/release");
    if custom_target.is_dir() {
        if let Some(binary) = find_binary_in_dir(&custom_target) {
            return Some(binary);
        }
    }

    // 通常のrelease/を探す
    let release_dir = target_dir.join("release");
    if release_dir.is_dir() {
        if let Some(binary) = find_binary_in_dir(&release_dir) {
            return Some(binary);
        }
    }

    None
}

fn find_binary_in_dir(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name()?.to_string_lossy();
            // 実行可能ファイルっぽいものを探す（拡張子なし、.so, .dなどを除外）
            if !filename.starts_with("lib")
                && !filename.ends_with(".d")
                && !filename.ends_with(".rlib")
                && !filename.ends_with(".so")
                && !filename.contains(".") {
                return Some(path);
            }
        }
    }

    None
}

fn emit_rerun_if_changed(path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        } else if metadata.is_dir() {
            println!("cargo:rerun-if-changed={}", path.display());
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    emit_rerun_if_changed(&entry.path());
                }
            }
        }
    }
}
