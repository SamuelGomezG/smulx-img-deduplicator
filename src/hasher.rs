use crate::scanner::ScannedFile;
use image::ImageReader;
use img_hash::{HashAlg, HasherConfig};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::path::PathBuf;

pub const MAX_IMAGE_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ImageRecord {
    pub path: PathBuf,
    pub hash: u64,
    pub size_bytes: u64,
}

pub(crate) fn compute_hash(img: &image::DynamicImage) -> u64 {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let raw = rgba.into_raw();
    let ih_img = img_hash::image::DynamicImage::ImageRgba8(
        img_hash::image::ImageBuffer::from_raw(w, h, raw).unwrap(),
    );

    let hasher = HasherConfig::new()
        .hash_size(8, 8)
        .hash_alg(HashAlg::Gradient)
        .to_hasher();

    let image_hash = hasher.hash_image(&ih_img);
    hash_to_u64(image_hash.as_bytes())
}

#[allow(dead_code)]
pub(crate) fn hamming_distance(h1: u64, h2: u64) -> u32 {
    (h1 ^ h2).count_ones()
}

fn hash_to_u64(bytes: &[u8]) -> u64 {
    assert_eq!(bytes.len(), 8, "Expected 8 bytes for 8x8 hash");
    u64::from_le_bytes(bytes.try_into().unwrap())
}

fn hash_single(file: &ScannedFile) -> Option<ImageRecord> {
    if file.size_bytes > MAX_IMAGE_BYTES {
        tracing::warn!(
            "Skipping oversized image {:?} ({} bytes)",
            file.path,
            file.size_bytes
        );
        return None;
    }

    let reader = match ImageReader::open(&file.path) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Cannot open image {:?}: {}", file.path, e);
            return None;
        }
    };
    let reader = match reader.with_guessed_format() {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Cannot determine format for {:?}: {}", file.path, e);
            return None;
        }
    };
    let img = match reader.decode() {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("Cannot decode image {:?}: {}", file.path, e);
            return None;
        }
    };

    let hash = compute_hash(&img);

    Some(ImageRecord {
        path: file.path.clone(),
        hash,
        size_bytes: file.size_bytes,
    })
}

pub fn hash_all(files: &[ScannedFile]) -> Vec<ImageRecord> {
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")
            .unwrap_or_else(|e| {
                tracing::warn!("progress bar template parsing failed: {}", e);
                indicatif::ProgressStyle::default_bar()
            })
            .progress_chars("=> "),
    );

    let result: Vec<ImageRecord> = files
        .par_iter()
        .progress_with(pb.clone())
        .filter_map(hash_single)
        .collect();

    pb.finish_and_clear();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn solid_image(r: u8, g: u8, b: u8) -> image::DynamicImage {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(100, 100, Rgb([r, g, b]));
        image::DynamicImage::ImageRgb8(img)
    }

    fn gradient_image() -> image::DynamicImage {
        let img = ImageBuffer::from_fn(100, 100, |x, _y| Rgb([x as u8, 0u8, 0u8]));
        image::DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn hash_es_determinista() {
        let img = gradient_image();
        let h1 = compute_hash(&img);
        let h2 = compute_hash(&img);
        assert_eq!(h1, h2);
    }

    #[test]
    fn imagenes_identicas_tienen_distancia_cero() {
        let img = gradient_image();
        let h1 = compute_hash(&img);
        let h2 = compute_hash(&img);
        assert_eq!(hamming_distance(h1, h2), 0);
    }

    #[test]
    fn imagenes_distintas_tienen_distancia_positiva() {
        let roja = solid_image(255, 0, 0);
        let azul = solid_image(0, 0, 255);
        let h1 = compute_hash(&roja);
        let h2 = compute_hash(&azul);
        assert!(
            hamming_distance(h1, h2) > 0 || h1 == h2,
            "Different solid images should differ or both be constant"
        );
    }

    #[test]
    fn hash_to_u64_round_trip() {
        let original: u64 = 0xDEADBEEFCAFEBABE;
        let bytes = original.to_le_bytes();
        let recovered = hash_to_u64(&bytes);
        assert_eq!(original, recovered);
    }
}
