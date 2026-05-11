use jwalk::WalkDir;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub size_bytes: u64,
}

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "tiff", "tif", "bmp"];

pub fn discover_images(roots: &[PathBuf]) -> Vec<ScannedFile> {
    let mut files = Vec::new();
    for root in roots {
        if !root.exists() {
            tracing::warn!("Root path does not exist: {:?}", root);
            continue;
        }
        if !root.is_dir() {
            tracing::warn!("Root path is not a directory: {:?}", root);
            continue;
        }
        let walker = WalkDir::new(root).follow_links(false).min_depth(1);

        for entry in walker {
            match entry {
                Ok(e) if e.file_type().is_file() => {
                    let path = e.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        && SUPPORTED_EXTENSIONS
                            .iter()
                            .any(|s| s.eq_ignore_ascii_case(ext))
                    {
                        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                        files.push(ScannedFile {
                            path,
                            size_bytes: size,
                        });
                    }
                }
                Err(e) => tracing::warn!("Error accessing entry: {}", e),
                _ => {}
            }
        }
    }
    files
}
