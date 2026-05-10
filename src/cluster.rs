use crate::bktree::BKTree;
use crate::hasher::ImageRecord;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ClusterFile {
    pub path: PathBuf,
    pub distance: u32,
    pub size_bytes: u64,
    pub marked_for_deletion: bool,
}

#[derive(Debug, Clone)]
pub struct ImageCluster {
    pub root_hash: u64,
    pub files: Vec<ClusterFile>,
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry] = rx;
                self.rank[rx] += 1;
            }
        }
    }
}

pub fn build_clusters(records: &[ImageRecord], threshold: u32) -> Vec<ImageCluster> {
    if records.is_empty() {
        return vec![];
    }

    let mut tree = BKTree::new();
    for record in records {
        tree.insert(record.hash, record.path.clone());
    }

    let mut path_to_index: HashMap<&std::path::Path, usize> = HashMap::with_capacity(records.len());
    for (i, record) in records.iter().enumerate() {
        path_to_index.insert(record.path.as_path(), i);
    }

    let mut uf = UnionFind::new(records.len());

    for (i, record) in records.iter().enumerate() {
        let neighbors = tree.search(record.hash, threshold);
        for (_, paths) in &neighbors {
            for path in *paths {
                if let Some(&j) = path_to_index.get(path.as_path()) {
                    uf.union(i, j);
                }
            }
        }
    }

    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..records.len() {
        let root = uf.find(i);
        components.entry(root).or_default().push(i);
    }

    let mut clusters: Vec<ImageCluster> = components
        .into_values()
        .filter(|group| group.len() >= 2)
        .map(|group| {
            let root_idx = group[0];
            let root_hash = records[root_idx].hash;
            let files = group
                .iter()
                .map(|&i| {
                    let r = &records[i];
                    ClusterFile {
                        path: r.path.clone(),
                        distance: hamming_distance(root_hash, r.hash),
                        size_bytes: r.size_bytes,
                        marked_for_deletion: false,
                    }
                })
                .collect();
            ImageCluster { root_hash, files }
        })
        .collect();

    clusters.sort_by(|a, b| b.files.len().cmp(&a.files.len()));

    clusters
}

pub(crate) fn hamming_distance(h1: u64, h2: u64) -> u32 {
    (h1 ^ h2).count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(path: &str, hash: u64) -> ImageRecord {
        ImageRecord {
            path: PathBuf::from(path),
            hash,
            size_bytes: 1024,
        }
    }

    #[test]
    fn sin_similitudes_no_produce_clusters() {
        let records = vec![
            record("a.jpg", 0x0000_0000_0000_0000),
            record("b.jpg", 0xFFFF_FFFF_FFFF_FFFF),
            record("c.jpg", 0xAAAA_AAAA_AAAA_AAAA),
        ];
        let clusters = build_clusters(&records, 5);
        assert!(
            clusters.is_empty(),
            "No debe haber clusters when todas las imagenes son distintas"
        );
    }

    #[test]
    fn duplicados_exactos_forman_cluster() {
        let records = vec![record("a.jpg", 42), record("b.jpg", 42)];
        let clusters = build_clusters(&records, 0);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files.len(), 2);
    }

    #[test]
    fn similares_dentro_de_umbral_forman_cluster() {
        let records = vec![
            record("a.jpg", 0b0000),
            record("b.jpg", 0b0001),
            record("c.jpg", 0b0011),
        ];
        let clusters = build_clusters(&records, 2);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files.len(), 3);
    }

    #[test]
    fn transitividad_une_cadena() {
        let a: u64 = 0b0000_0000;
        let b: u64 = 0b0000_0011;
        let c: u64 = 0b0000_1111;
        let records = vec![record("a.jpg", a), record("b.jpg", b), record("c.jpg", c)];
        let clusters = build_clusters(&records, 2);
        assert_eq!(
            clusters.len(),
            1,
            "La transitividad debe unir a, b y c en un solo cluster"
        );
        assert_eq!(clusters[0].files.len(), 3);
    }

    #[test]
    fn grupos_independientes_no_se_mezclan() {
        let records = vec![
            record("a.jpg", 0x0000_0000_0000_0000),
            record("b.jpg", 0x0000_0000_0000_0001),
            record("c.jpg", 0xFFFF_FFFF_FFFF_FFFF),
            record("d.jpg", 0xFFFF_FFFF_FFFF_FFFE),
        ];
        let clusters = build_clusters(&records, 3);
        assert_eq!(clusters.len(), 2);
        assert!(clusters.iter().all(|c| c.files.len() == 2));
    }

    #[test]
    fn singleton_no_aparece_en_resultado() {
        let records = vec![
            record("a.jpg", 0x0000),
            record("b.jpg", 0x0001),
            record("c.jpg", 0xFFFF),
        ];
        let clusters = build_clusters(&records, 2);
        assert_eq!(clusters.len(), 1);
        let paths: Vec<_> = clusters[0].files.iter().map(|f| f.path.as_path()).collect();
        assert!(!paths.contains(&std::path::Path::new("c.jpg")));
    }
}
