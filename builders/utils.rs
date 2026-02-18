use std::fs;
use std::path::Path;

/// ディレクトリとその中身を再帰的に監視対象に追加
pub fn emit_rerun_if_changed(path: &Path) {
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

/// ディレクトリ内で実行ファイルらしきものを探す
pub fn find_binary_in_dir(dir: &Path) -> Option<std::path::PathBuf> {
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
                && !filename.contains('.')
            {
                return Some(path);
            }
        }
    }

    None
}

/// カスタムターゲット仕様ファイルを探す
pub fn find_target_spec(app_dir: &Path) -> Option<String> {
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

    None
}
