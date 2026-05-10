use jwalk::WalkDir;
use std::path::PathBuf;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "tiff", "tif", "bmp"];

pub fn discover_images(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for root in roots {
        let walker = WalkDir::new(root).follow_links(false).min_depth(1);

        for entry in walker {
            match entry {
                Ok(e) if e.file_type().is_file() => {
                    let path = e.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        && SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
                    {
                        paths.push(path);
                    }
                }
                Err(e) => tracing::warn!("Error accessing entry: {}", e),
                _ => {}
            }
        }
    }
    paths
}
