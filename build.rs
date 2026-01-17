use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let initfs_dir_core = manifest_dir.join("src/core/init/initfs");
    let initfs_dir_legacy = manifest_dir.join("src/init/initfs");
    let initfs_dir = if initfs_dir_core.is_dir() {
        initfs_dir_core
    } else {
        initfs_dir_legacy
    };

    if !initfs_dir.is_dir() {
        panic!("initfs directory not found at {:?}", initfs_dir);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let image_path = out_dir.join("initfs.ext2");

    emit_rerun_if_changed(&initfs_dir);

    let status = Command::new("mke2fs")
        .args([
            "-t",
            "ext2",
            "-b",
            "4096",
            "-m",
            "0",
            "-L",
            "initfs",
            "-d",
        ])
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
            panic!(
                "failed to execute mke2fs: {e}. Please install e2fsprogs (mke2fs)."
            );
        }
    }
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
