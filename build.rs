use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use num_cpus;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // fsディレクトリ
    let fs_dir = manifest_dir.join("fs");

    // fsディレクトリが存在しない場合、作成
    if !fs_dir.is_dir() {
        fs::create_dir_all(&fs_dir).expect(&format!(
            "Failed to create initfs directory: {}",
            fs_dir.display()
        ));
    }

    // newlibのビルド
    let newlib_src_dir = manifest_dir.join("src/lib");
    if !newlib_src_dir.exists() {
         panic!("Newlib source not found at {}", newlib_src_dir.display());
    }
    build_newlib(&newlib_src_dir);

    let target = env::var("TARGET").unwrap_or("x86_64-unknown-uefi".to_string());
    let profile = env::var("PROFILE").unwrap_or("debug".to_string());
    let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string()));

    let abs_target_dir = if target_dir.is_absolute() {
        target_dir
    } else {
        manifest_dir.join(target_dir)
    };

    let newlib_install_dir = abs_target_dir
        .join(&target)
        .join(&profile)
        .join("newlib_install");

    // libc.a, libg.a, libm.a, libnosys.aをinitfsにコピー
    let libc_dir = newlib_install_dir
        .join("x86_64-elf")
        .join("lib");

    // ユーザーライブラリをinitfsにコピー
    let user_src_dir = manifest_dir.join("src/user");
    build_user_libs(&user_src_dir, &libc_dir);

    // crt0.oをコピー
    let crt0_src = libc_dir.join("crt0.o");
    let crt0_dest = fs_dir.join("crt0.o");
    if let Err(e) = fs::copy(&crt0_src, &crt0_dest) {
        panic!("Failed to copy crt0.o to initfs: {}", e);
    } else {
        println!("Copied crt0.o to initfs");
    }

    let libs = ["libc.a", "libg.a", "libm.a", "libnosys.a"];
    for lib in &libs {
        let src = libc_dir.join(lib);
        let dest = fs_dir.join(lib);
        if let Err(e) = fs::copy(&src, &dest) {
            panic!("Failed to copy {} to initfs: {}. Make sure newlib is built correctly.", lib, e);
        } else {
            println!("Copied {} to initfs (from {})", lib, src.display());
        }
    }

    let apps_dir = manifest_dir.join("src/apps");
    let services_dir = manifest_dir.join("src/services");

    // appsディレクトリが存在する場合、アプリをビルド
    if apps_dir.is_dir() {
        build_apps(&apps_dir, &fs_dir, "elf");
    }

    // servicesディレクトリが存在する場合、サービスをビルド
    if services_dir.is_dir() {
        build_apps(&services_dir, &fs_dir, "service");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let image_path = out_dir.join("initfs.ext2");

    emit_rerun_if_changed(&fs_dir);

    let status = Command::new("mke2fs")
        .args(["-t", "ext2", "-b", "4096", "-m", "0", "-L", "initfs", "-d"])
        .arg(&fs_dir)
        .arg(&image_path)
        .arg("32768") // 128MB
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
    
    let mkimage_script = manifest_dir.join("scripts/make_image.sh");
    let _ = Command::new(mkimage_script).status();
}

fn build_newlib(src_dir: &Path) {
    let target = env::var("TARGET").expect("TARGET not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");

    let target_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string()));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Resolve absolute target dir
    let abs_target_dir = if target_dir.is_absolute() {
        target_dir
    } else {
        manifest_dir.join(target_dir)
    };

    let build_base_dir = abs_target_dir
        .join(&target)
        .join(&profile);

    let install_dir = build_base_dir.join("newlib_install");
    let build_dir = build_base_dir.join("newlib_build");

    // Check if libc.a exists in the install location
    if install_dir.join("x86_64-elf/lib/libc.a").exists() {
        println!("newlib already built, skipping");
        return;
    }

    if !build_dir.exists() {
        fs::create_dir_all(&build_dir).expect("Failed to create newlib build dir");
    }

    // Configure (if Makefile doesn't exist)
    if !build_dir.join("Makefile").exists() {
        println!("Configuring newlib...");

        let configure_script = src_dir.join("configure");
        if !configure_script.exists() {
            panic!("configure script not found at {}", configure_script.display());
        }

        // We need an absolute path to configure script usually
        let abs_configure = configure_script.canonicalize().unwrap();

        let status = Command::new(abs_configure)
            .current_dir(&build_dir)
            .arg(format!("--target={}", "x86_64-elf"))
            .arg(format!("--prefix={}", install_dir.display()))
            .arg("--disable-multilib")
            .status()
            .expect("Failed to execute newlib configure");

        if !status.success() {
            let _ = fs::remove_dir_all(&build_dir);
            panic!("Newlib configure failed. Build directory cleaned up.");
        }
    }

    let cpu_cores = num_cpus::get();
    let make_j = format!("-j{}", cpu_cores);

    println!("Building newlib...");

    let status = Command::new("make")
        .current_dir(&build_dir)
        .arg(make_j)
        .status()
        .expect("Failed to execute newlib make");

    if !status.success() {
        let _ = fs::remove_dir_all(&build_dir);
        panic!("Newlib make failed. Build directory cleaned up. Please try again.");
    }

    println!("Installing newlib...");

    let status = Command::new("make")
        .current_dir(&build_dir)
        .arg("install")
        .status()
        .expect("Failed to execute newlib make install");

    if !status.success() {
        let _ = fs::remove_dir_all(&build_dir);
        panic!("Newlib make install failed. Build directory cleaned up.");
    }
}

fn build_user_libs(user_dir: &Path, libc_dir: &Path) {
    println!("Building user libs...");

    if !libc_dir.exists() {
        fs::create_dir_all(libc_dir).expect("Failed to create libc dir");
    }

    let crt_src = user_dir.join("crt.rs");
    let crt_obj = libc_dir.join("crt0.o");

    let status = Command::new("rustc")
        .args(&["--emit", "obj"])
        .args(&["--crate-type", "lib"])
        .args(&["--edition", "2021"])
        .args(&["--target", "x86_64-unknown-none"])
        .args(&["-o", crt_obj.to_str().unwrap()])
        .arg(&crt_src)
        .status()
        .expect("Failed to build crt0.o");

    if !status.success() {
        panic!("Failed to build crt0.o");
    }

    // 2. libuserglue.a のビルド
    let lib_src = user_dir.join("lib.rs");
    let glue_lib = libc_dir.join("libuserglue.a");

    let status = Command::new("rustc")
        .args(&["--crate-type", "staticlib"])
        .args(&["--edition", "2021"])
        .args(&["--target", "x86_64-unknown-none"])
        .args(&["-C", "panic=abort"])
        .args(&["-o", glue_lib.to_str().unwrap()])
        .arg(&lib_src)
        .status()
        .expect("Failed to build libuserglue.a");

    if !status.success() {
        panic!("Failed to build libuserglue.a");
    }

    // 3. libc.a にマージ
    // 作業用ディレクトリを作る
    let merge_dir = libc_dir.join("merge_tmp");
    if merge_dir.exists() {
        fs::remove_dir_all(&merge_dir).unwrap();
    }
    fs::create_dir(&merge_dir).unwrap();

    // libc.a, libuserglue.a を絶対パスで取得しておく
    let libc_a = libc_dir.join("libc.a");
    let libglue_a = glue_lib;

    // ar x libc.a
    let status = Command::new("ar")
        .current_dir(&merge_dir)
        .arg("x")
        .arg(&libc_a)
        .status()
        .expect("Failed to extract libc.a");
    if !status.success() { panic!("ar x libc.a failed"); }

    // ar x libuserglue.a
    let status = Command::new("ar")
        .current_dir(&merge_dir)
        .arg("x")
        .arg(&libglue_a)
        .status()
        .expect("Failed to extract libuserglue.a");
    if !status.success() { panic!("ar x libuserglue.a failed"); }

    // ar rcs libc.a *.o
    let status = Command::new("sh")
        .current_dir(&merge_dir)
        .arg("-c")
        .arg(format!("ar rcs {} *.o", libc_a.to_str().unwrap()))
        .status()
        .expect("Failed to repack libc.a");

    if !status.success() { panic!("ar rcs libc.a failed"); }

    // クリーンアップ
    fs::remove_dir_all(&merge_dir).unwrap();
    println!("Successfully merged user glue into libc.a");
}

fn build_apps(apps_dir: &Path, initfs_dir: &Path, extension: &str) {
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
        println!("Building app: {}", app_name);

        // アプリのソースファイルを明示的に監視
        println!("cargo:rerun-if-changed={}", cargo_toml.display());
        let src_dir = path.join("src");
        if src_dir.is_dir() {
            emit_rerun_if_changed(&src_dir);
        }

        // カスタムターゲットファイルを探す
        let target_spec = find_target_spec(&path);

        // cargoでアプリをビルド（出力をキャプチャ）
        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--release"]);

        // カスタムターゲットが見つかった場合は指定
        if let Some(target) = &target_spec {
            cmd.arg("--target").arg(target);
            println!("  Using target: {}", target);
        } else {
            // デフォルトは ELF (for newlib)
            let default_target = "x86_64-unknown-none";
            cmd.arg("--target").arg(default_target);
            println!("  Using default target: {}", default_target);
        }

        let output = cmd.current_dir(&path).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // ビルド成果物を探す
                    let target_dir = path.join("target");
                    let target_name = if let Some(p) = &target_spec {
                         Path::new(p).file_stem().map(|s| s.to_string_lossy().to_string())
                    } else {
                         Some("x86_64-unknown-none".to_string())
                    };

                    if let Some(elf_path) = find_built_binary(&target_dir, target_name.as_deref()) {
                        let dest_name = format!("{}.{}", app_name, extension);
                        let dest = initfs_dir.join(&dest_name);
                        if let Err(e) = fs::copy(&elf_path, &dest) {
                            println!("cargo:warning=Failed to copy {} to initfs: {}", dest_name, e);
                        } else {
                            println!("Copied {} to initfs (from {})", dest_name, elf_path.display());
                        }
                    } else {
                        println!("cargo:warning=Built binary not found for {}", app_name);
                    }
                } else {
                    println!("cargo:warning=Failed to build app: {}", app_name);
                    // エラー出力を表示
                    if !output.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        for line in stderr.lines().take(10) {
                            println!("cargo:warning=  {}", line);
                        }
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute cargo for {}: {}", app_name, e);
            }
        }
    }
}

fn find_built_binary(target_dir: &Path, target_name: Option<&str>) -> Option<PathBuf> {
    // カスタムターゲットが指定されている場合はそのディレクトリを優先
    if let Some(target) = target_name {
        let custom_target = target_dir.join(format!("{}/release", target));
        if custom_target.is_dir() {
            if let Some(binary) = find_binary_in_dir(&custom_target) {
                return Some(binary);
            }
        }
    }

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

fn find_target_spec(app_dir: &Path) -> Option<String> {
    // .jsonファイルを探す（x86_64-*.json など）
    if let Ok(entries) = fs::read_dir(app_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".json") && filename_str.starts_with("x86_64-") {
                        // 絶対パスを返す
                        return path.to_str().map(|s| s.to_string());
                    }
                }
            }
        }
    }

    // .cargo/config.tomlでデフォルトターゲットが指定されている可能性もあるが、
    // とりあえずjsonファイルの検出のみ
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
    // targetディレクトリは除外
    if let Some(file_name) = path.file_name() {
        if file_name == "target" || file_name == ".git" {
            return;
        }
    }

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
