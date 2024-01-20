use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;

struct FileInfo {
    name: String,
    extension: String,
}

impl FileInfo {
    fn new(path: &Path) -> io::Result<FileInfo> {
        let name = path
            .file_stem()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Missing file stem in {:?}", path),
                )
            })?
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Non-unicode file stem in {:?}", path),
                )
            })?
            .to_string();
        let extension = path
            .extension()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Missing file extension in {:?}", path),
                )
            })?
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Non-unicode file extension in {:?}", path),
                )
            })?
            .to_string();
        Ok(FileInfo { name, extension })
    }
}

fn is_excluded_file(name: &str, extension: &str) -> bool {
    (name == "extension-sorter" && extension == "exe")
        || (name == "extension-sorter-log" && extension == "txt")
}

fn main() -> io::Result<()> {
    let mut log_file = fs::File::create("extension-sorter-log.txt")?;
    let mut result: HashMap<String, Vec<FileInfo>> = HashMap::new();

    for entry in fs::read_dir(".")? {
        let entry: fs::DirEntry = entry?;
        let path: std::path::PathBuf = entry.path();
        if path.is_file() {
            match FileInfo::new(&path) {
                Ok(file_info) => {
                    if !is_excluded_file(&file_info.name, &file_info.extension) {
                        if !Path::new(&file_info.extension).exists() {
                            fs::create_dir(&file_info.extension)?;
                        }
                        let new_path = format!(
                            "{}/{}.{}",
                            file_info.extension, file_info.name, file_info.extension
                        );
                        fs::rename(path, new_path)?;
                        result
                            .entry(file_info.extension.clone())
                            .or_default()
                            .push(file_info);
                    }
                }
                Err(e) => {
                    if let Err(e) = writeln!(log_file, "Error processing file {:?}: {}", path, e) {
                        eprintln!("Failed to write to log file : {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }

    let mut extensions: Vec<_> = result.keys().collect();
    extensions.sort_by_key(|a: &&String| a.to_lowercase());

    let mut result_file: fs::File = fs::File::create("result.md")?;
    writeln!(result_file, "# File move result\n")?;
    for extension in extensions {
        writeln!(result_file, "## {}\n", extension)?;
        for file_info in &result[extension] {
            writeln!(
                result_file,
                "- {}.{}\n",
                file_info.name, file_info.extension
            )?;
        }
    }
    writeln!(log_file, "Program completed successfully")?;
    Ok(())
}
