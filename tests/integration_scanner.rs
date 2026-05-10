use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn touch(path: &PathBuf) {
    fs::write(path, b"fake-image-data").unwrap();
}

#[test]
fn discover_images_finds_supported_extensions() {
    let dir = TempDir::new().unwrap();
    touch(&dir.path().join("a.jpg"));
    touch(&dir.path().join("b.png"));
    touch(&dir.path().join("c.webp"));
    touch(&dir.path().join("d.gif"));

    let paths = smulx_img_deduplicator::scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(paths.len(), 4);
}

#[test]
fn discover_images_ignores_unsupported_extensions() {
    let dir = TempDir::new().unwrap();
    touch(&dir.path().join("photo.jpg"));
    touch(&dir.path().join("document.pdf"));
    touch(&dir.path().join("script.py"));
    touch(&dir.path().join("archive.zip"));

    let paths = smulx_img_deduplicator::scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("photo.jpg"));
}

#[test]
fn discover_images_traverses_nested_directories() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("subdir");
    let nested = sub.join("nested");
    fs::create_dir_all(&nested).unwrap();
    touch(&dir.path().join("root.jpg"));
    touch(&sub.join("sub.png"));
    touch(&nested.join("deep.webp"));

    let paths = smulx_img_deduplicator::scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(paths.len(), 3);
}

#[test]
fn discover_images_empty_directory_returns_empty() {
    let dir = TempDir::new().unwrap();
    let paths = smulx_img_deduplicator::scanner::discover_images(&[dir.path().to_path_buf()]);
    assert!(paths.is_empty());
}

#[test]
fn discover_images_multiple_roots() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    touch(&dir1.path().join("a.jpg"));
    touch(&dir2.path().join("b.png"));

    let roots = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
    let paths = smulx_img_deduplicator::scanner::discover_images(&roots);
    assert_eq!(paths.len(), 2);
}

#[test]
fn discover_images_skips_nonexistent_root() {
    let dir = TempDir::new().unwrap();
    touch(&dir.path().join("photo.jpg"));
    let bad: PathBuf = PathBuf::from("/nonexistent/path");

    let roots = vec![bad, dir.path().to_path_buf()];
    let paths = smulx_img_deduplicator::scanner::discover_images(&roots);
    assert_eq!(paths.len(), 1);
}

#[test]
fn discover_images_skips_non_directory_root() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("not_a_dir.jpg");
    touch(&file);

    let paths = smulx_img_deduplicator::scanner::discover_images(&[file]);
    assert!(paths.is_empty(), "A file passed as root should be skipped");
}

#[test]
fn discover_images_case_insensitive_extension() {
    let dir = TempDir::new().unwrap();
    touch(&dir.path().join("photo.JPG"));
    touch(&dir.path().join("photo.PNG"));
    touch(&dir.path().join("photo.TIFF"));

    let paths = smulx_img_deduplicator::scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(paths.len(), 3);
}
