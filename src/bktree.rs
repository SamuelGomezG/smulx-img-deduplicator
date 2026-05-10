use std::collections::HashMap;
use std::path::PathBuf;

pub struct BKNode {
    pub(crate) hash: u64,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) children: HashMap<u8, Box<BKNode>>,
}

impl BKNode {
    pub fn new(hash: u64, path: PathBuf) -> Self {
        BKNode {
            hash,
            paths: vec![path],
            children: HashMap::new(),
        }
    }
}

pub struct BKTree {
    root: Option<Box<BKNode>>,
}

impl Default for BKTree {
    fn default() -> Self {
        Self::new()
    }
}

impl BKTree {
    pub fn new() -> Self {
        BKTree { root: None }
    }

    pub fn insert(&mut self, hash: u64, path: PathBuf) {
        match &mut self.root {
            None => {
                self.root = Some(Box::new(BKNode::new(hash, path)));
            }
            Some(root) => {
                insert_node(root, hash, path);
            }
        }
    }

    pub fn search(&self, query: u64, threshold: u32) -> Vec<(u64, &[PathBuf])> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            search_node(root, query, threshold, &mut results);
        }
        results
    }
}

pub(crate) fn hamming_distance(h1: u64, h2: u64) -> u32 {
    (h1 ^ h2).count_ones()
}

pub(crate) fn insert_node(node: &mut BKNode, hash: u64, path: PathBuf) {
    let d = hamming_distance(node.hash, hash) as u8;

    if d == 0 {
        node.paths.push(path);
        return;
    }

    match node.children.get_mut(&d) {
        Some(child) => insert_node(child, hash, path),
        None => {
            node.children.insert(d, Box::new(BKNode::new(hash, path)));
        }
    }
}

pub(crate) fn search_node<'a>(
    node: &'a BKNode,
    query: u64,
    threshold: u32,
    results: &mut Vec<(u64, &'a [PathBuf])>,
) {
    let d = hamming_distance(node.hash, query);

    if d <= threshold {
        results.push((node.hash, &node.paths));
    }

    let lo = d.saturating_sub(threshold) as u8;
    let hi = (d + threshold).min(64) as u8;

    for dist_key in lo..=hi {
        if let Some(child) = node.children.get(&dist_key) {
            search_node(child, query, threshold, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn insertar_y_buscar_exacto() {
        let mut tree = BKTree::new();
        tree.insert(0b0000_0000u64, p("img_a.jpg"));
        let results = tree.search(0b0000_0000u64, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1[0], p("img_a.jpg"));
    }

    #[test]
    fn duplicado_exacto_acumula_paths_en_mismo_nodo() {
        let mut tree = BKTree::new();
        tree.insert(42u64, p("a.jpg"));
        tree.insert(42u64, p("b.jpg"));
        let results = tree.search(42u64, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.len(), 2);
    }

    #[test]
    fn busqueda_con_umbral_encuentra_vecinos() {
        let mut tree = BKTree::new();
        tree.insert(0u64, p("exacto.jpg"));
        tree.insert(1u64, p("cercano.jpg"));
        tree.insert(255u64, p("lejano.jpg"));

        let results = tree.search(0u64, 2);
        let all_paths: Vec<&PathBuf> = results.iter().flat_map(|(_, ps)| ps.iter()).collect();

        assert!(all_paths.contains(&&p("exacto.jpg")));
        assert!(all_paths.contains(&&p("cercano.jpg")));
        assert!(!all_paths.contains(&&p("lejano.jpg")));
    }

    #[test]
    fn busqueda_umbral_cero_solo_exactos() {
        let mut tree = BKTree::new();
        tree.insert(0u64, p("exacto.jpg"));
        tree.insert(1u64, p("diferente.jpg"));

        let results = tree.search(0u64, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1[0], p("exacto.jpg"));
    }

    #[test]
    fn tree_vacio_devuelve_vec_vacio() {
        let tree = BKTree::new();
        let results = tree.search(12345u64, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn desigualdad_triangular_no_omite_resultados() {
        let mut tree = BKTree::new();
        let base: u64 = 0b1111_0000_0000_0000u64;
        for i in 0u64..64 {
            tree.insert(
                base ^ (1u64 << i.min(63)),
                PathBuf::from(format!("img_{}.jpg", i)),
            );
        }
        let results = tree.search(base, 3);
        for (hash, _) in &results {
            assert!((base ^ hash).count_ones() <= 3);
        }
    }
}
