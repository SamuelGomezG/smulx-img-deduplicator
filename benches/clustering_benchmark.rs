use criterion::{Criterion, criterion_group, criterion_main};
use smulx_img_deduplicator::bktree::BKTree;
use smulx_img_deduplicator::hasher::ImageRecord;
use std::collections::HashMap;
use std::hint::black_box;
use std::path::PathBuf;

fn hamming_distance(h1: u64, h2: u64) -> u32 {
    (h1 ^ h2).count_ones()
}

fn generate_records(num_groups: usize, per_group: usize, gap: u64) -> Vec<ImageRecord> {
    let mut records = Vec::with_capacity(num_groups * per_group);
    for g in 0..num_groups {
        let base_hash = (g as u64) * gap;
        for m in 0..per_group {
            let hash = base_hash + m as u64;
            records.push(ImageRecord {
                path: PathBuf::from(format!("/tmp/group{}_{}.jpg", g, m)),
                hash,
                size_bytes: 1024,
            });
        }
    }
    records
}

fn build_clusters_linear(
    records: &[ImageRecord],
    threshold: u32,
) -> Vec<smulx_img_deduplicator::cluster::ImageCluster> {
    if records.is_empty() {
        return vec![];
    }

    let mut tree = BKTree::new();
    for record in records {
        tree.insert(record.hash, record.path.clone());
    }

    let mut uf = UnionFind::new(records.len());

    for (i, record) in records.iter().enumerate() {
        let neighbors = tree.search(record.hash, threshold);
        for (_, paths) in &neighbors {
            for path in *paths {
                if let Some(j) = records.iter().position(|r| &r.path == path) {
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

    let mut clusters: Vec<smulx_img_deduplicator::cluster::ImageCluster> = components
        .into_values()
        .filter(|group| group.len() >= 2)
        .map(|group| {
            let root_idx = group[0];
            let root_hash = records[root_idx].hash;
            let files = group
                .iter()
                .map(|&i| {
                    let r = &records[i];
                    smulx_img_deduplicator::cluster::ClusterFile {
                        path: r.path.clone(),
                        distance: hamming_distance(root_hash, r.hash),
                        size_bytes: r.size_bytes,
                        marked_for_deletion: false,
                    }
                })
                .collect();
            smulx_img_deduplicator::cluster::ImageCluster { root_hash, files }
        })
        .collect();

    clusters.sort_by(|a, b| b.files.len().cmp(&a.files.len()));
    clusters
}

fn build_clusters_hashmap(
    records: &[ImageRecord],
    threshold: u32,
) -> Vec<smulx_img_deduplicator::cluster::ImageCluster> {
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

    let mut clusters: Vec<smulx_img_deduplicator::cluster::ImageCluster> = components
        .into_values()
        .filter(|group| group.len() >= 2)
        .map(|group| {
            let root_idx = group[0];
            let root_hash = records[root_idx].hash;
            let files = group
                .iter()
                .map(|&i| {
                    let r = &records[i];
                    smulx_img_deduplicator::cluster::ClusterFile {
                        path: r.path.clone(),
                        distance: hamming_distance(root_hash, r.hash),
                        size_bytes: r.size_bytes,
                        marked_for_deletion: false,
                    }
                })
                .collect();
            smulx_img_deduplicator::cluster::ImageCluster { root_hash, files }
        })
        .collect();

    clusters.sort_by(|a, b| b.files.len().cmp(&a.files.len()));
    clusters
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

fn benchmark(c: &mut Criterion) {
    let sizes = [100usize, 500, 1000, 3000];

    for &size in &sizes {
        let records = generate_records(size / 5, 5, 3);

        let group_name = format!("cluster_{}_records", size);
        let mut group = c.benchmark_group(&group_name);

        group.bench_function("linear_scan", |b| {
            b.iter(|| {
                let result = build_clusters_linear(black_box(&records), black_box(5));
                black_box(result)
            })
        });

        group.bench_function("hashmap_lookup", |b| {
            b.iter(|| {
                let result = build_clusters_hashmap(black_box(&records), black_box(5));
                black_box(result)
            })
        });

        group.finish();
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
