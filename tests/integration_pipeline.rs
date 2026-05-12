use image::{ImageBuffer, Rgb};
use smulx_img_deduplicator::{cluster, hasher, scanner};
use std::path::PathBuf;
use tempfile::TempDir;

fn write_solid_image(dir: &TempDir, name: &str, r: u8, g: u8, b: u8) -> PathBuf {
    let path = dir.path().join(name);
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(200, 200, Rgb([r, g, b]));
    img.save(&path).unwrap();
    path
}

fn write_noisy_image(dir: &TempDir, name: &str, base_r: u8) -> PathBuf {
    let path = dir.path().join(name);
    let img = ImageBuffer::from_fn(200, 200, |x, y| {
        Rgb([base_r.saturating_add(((x + y) % 3) as u8), 0u8, 0u8])
    });
    img.save(&path).unwrap();
    path
}

#[test]
fn pipeline_agrupa_imagenes_similares_y_descarta_distintas() {
    let dir = TempDir::new().unwrap();

    write_solid_image(&dir, "roja_1.jpg", 255, 0, 0);
    write_noisy_image(&dir, "roja_2.jpg", 250);

    // Blue gradient image — distinct from uniform red
    let azul_path = dir.path().join("azul.jpg");
    let azul_img = ImageBuffer::from_fn(200, 200, |x, _y| Rgb([0u8, 0u8, x as u8]));
    azul_img.save(&azul_path).unwrap();

    let files = scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(files.len(), 3);

    let records = hasher::hash_all(&files);
    assert_eq!(
        records.len(),
        3,
        "Todas las imagenes deben hashearse correctamente"
    );

    let clusters = cluster::build_clusters(&records, 5);

    for c in &clusters {
        let filename_contains = |substring: &str| -> bool {
            c.files.iter().any(|f| {
                f.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.contains(substring))
                    .unwrap_or(false)
            })
        };
        assert!(
            !(filename_contains("roja") && filename_contains("azul")),
            "Las imagenes rojas y azul no deben estar en el mismo cluster"
        );
    }
}

#[test]
fn pipeline_duplicados_exactos_misma_imagen_mismo_cluster() {
    let dir = TempDir::new().unwrap();
    let original = write_solid_image(&dir, "original.png", 128, 64, 32);

    let copia = dir.path().join("copia.png");
    std::fs::copy(&original, &copia).unwrap();

    let files = scanner::discover_images(&[dir.path().to_path_buf()]);
    assert_eq!(files.len(), 2, "Both original and copy must be discovered");
    let records = hasher::hash_all(&files);
    let clusters = cluster::build_clusters(&records, 0);

    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].files.len(), 2);
    assert_eq!(clusters[0].files[0].distance, 0);
    assert_eq!(clusters[0].files[1].distance, 0);
}
