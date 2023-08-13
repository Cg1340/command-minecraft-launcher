use chrono::{DateTime, Local};
use colored::Colorize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str;
use uuid::Uuid;
use zip::read::ZipArchive;

pub mod post;
pub mod downloader;
pub mod minecraft_core;

/// 向指定的文件写入指定的内容。
///
/// `path`: 文件路径。
///
/// `contents`: 写入的内容。
///
/// ## examples
///
/// ```rust
/// use minecraft::write_to_file;
/// write_to_file("./.minecraft/assets/indexes/1.19.json", b"{}");
/// ```
///

pub fn write_to_file(path: &str, contents: &[u8]) {
    let _path = Path::new(path);
    let result = std::fs::create_dir_all(_path.parent().unwrap_or_else(|| _path));
    match result {
        Ok(_) => {}
        Err(err) => {
            panic!("写入 {} 文件时发生错误: {}", path, err);
        }
    }
    std::fs::write(_path, contents).unwrap();
}

/// 获取目标的绝对路径。
///
/// `path`: 目标路径。
///
/// return: 目标的绝对路径。
///
/// ## Example
///
/// ```rust
/// use minecraft::get_path;
/// println!("{}", get_path("./.minecraft").display().to_string());
/// ```
///

pub fn get_path(path: &str) -> PathBuf {
    let canonicalized_path = std::fs::canonicalize(path).unwrap();
    let stripped_path = strip_long_path_prefix(&canonicalized_path);
    PathBuf::from(stripped_path)
}

fn strip_long_path_prefix(path: &Path) -> String {
    let path_string = path.to_string_lossy().into_owned();
    if let Some(stripped) = path_string.strip_prefix(r"\\?\") {
        stripped.to_owned()
    } else {
        path_string
    }
}

pub fn info(info: &str) {
    let local: DateTime<Local> = Local::now();
    println!("[{}]#{} {}", local.format("%H:%M:%S"), "info".blue(), info);
}

/// 解压文件。
///
/// `file`: 要解压文件的路径。
///
/// `target`: 目标路径。
///

fn extract(file: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(file)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.name().to_owned();
        let target_path = target.join(&file_path);

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        if !file_path.ends_with('/') {
            if target_path.exists() {
                continue;
            }

            if let Ok(mut output_file) = File::create(&target_path) {
                std::io::copy(&mut file, &mut output_file)?;
            } else {
                return Err(format!("Error creating file: {}", target_path.display()).into());
            }
        }
    }

    Ok(())
}

pub fn generate_uuid_without_hyphens(input: &str) -> String {
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &input.as_bytes());
    let uuid_string = uuid.to_string();
    let uuid_without_hyphens = uuid_string.replace("-", "");
    uuid_without_hyphens
}
