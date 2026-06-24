use smulx_img_deduplicator::cluster::ImageCluster;

pub enum AppMode {
    ClusterList,
    FileList,
    ConfirmDelete,
    ErrorMessage(String),
}

pub struct App {
    pub clusters: Vec<ImageCluster>,
    pub selected_cluster: usize,
    pub selected_file: usize,
    pub mode: AppMode,
    pub status_message: String,
    pub use_trash: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(clusters: Vec<ImageCluster>, use_trash: bool) -> Self {
        App {
            clusters,
            selected_cluster: 0,
            selected_file: 0,
            mode: AppMode::ClusterList,
            status_message: String::from(
                "Tab: cambiar foco | Espacio: marcar | Enter: borrar marcados | q: salir",
            ),
            use_trash,
            should_quit: false,
        }
    }

    pub fn current_cluster(&self) -> Option<&ImageCluster> {
        self.clusters.get(self.selected_cluster)
    }

    pub fn current_cluster_mut(&mut self) -> Option<&mut ImageCluster> {
        self.clusters.get_mut(self.selected_cluster)
    }

    pub fn toggle_mark(&mut self) {
        let idx = self.selected_file;
        let is_currently_marked = self
            .clusters
            .get(self.selected_cluster)
            .and_then(|c| c.files.get(idx))
            .map(|f| f.marked_for_deletion)
            .unwrap_or(false);

        if !is_currently_marked {
            let already_marked = self
                .clusters
                .get(self.selected_cluster)
                .map(|c| c.files.iter().filter(|f| f.marked_for_deletion).count())
                .unwrap_or(0);
            let total = self
                .clusters
                .get(self.selected_cluster)
                .map(|c| c.files.len())
                .unwrap_or(0);
            if total.saturating_sub(already_marked) <= 1 {
                return;
            }
        }

        if let Some(cluster) = self.current_cluster_mut()
            && let Some(file) = cluster.files.get_mut(idx)
        {
            file.marked_for_deletion = !file.marked_for_deletion;
        }
    }

    pub fn execute_deletion(&mut self) -> (usize, Vec<String>) {
        let mut deleted = 0;
        let mut errors = Vec::new();
        let use_trash = self.use_trash;

        if let Some(cluster) = self.clusters.get_mut(self.selected_cluster) {
            cluster.files.retain(|f| {
                if !f.marked_for_deletion {
                    return true;
                }

                let result = if use_trash {
                    trash::delete(&f.path).map_err(|e| format!("{}", e))
                } else {
                    std::fs::remove_file(&f.path).map_err(|e| format!("{}", e))
                };

                match result {
                    Ok(()) => {
                        deleted += 1;
                        false
                    }
                    Err(msg) => {
                        errors.push(format!("{}: {}", f.path.display(), msg));
                        true
                    }
                }
            });

            if cluster.files.len() <= 1 {
                self.clusters.remove(self.selected_cluster);
                if self.selected_cluster >= self.clusters.len() && !self.clusters.is_empty() {
                    self.selected_cluster = self.clusters.len() - 1;
                }
                self.selected_file = 0;
            } else {
                self.selected_file = self
                    .selected_file
                    .min(cluster.files.len().saturating_sub(1));
            }
        }

        (deleted, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smulx_img_deduplicator::cluster::ClusterFile;
    use std::path::PathBuf;

    fn make_cluster(n: usize) -> ImageCluster {
        ImageCluster {
            root_hash: 0,
            files: (0..n)
                .map(|i| ClusterFile {
                    path: PathBuf::from(format!("/tmp/img_{}.jpg", i)),
                    distance: 0,
                    size_bytes: 1024 * i as u64,
                    marked_for_deletion: false,
                })
                .collect(),
        }
    }

    fn make_app(cluster_sizes: &[usize]) -> App {
        let clusters = cluster_sizes.iter().map(|&n| make_cluster(n)).collect();
        App::new(clusters, true)
    }

    #[test]
    fn toggle_mark_marca_archivo_seleccionado() {
        let mut app = make_app(&[3]);
        app.mode = AppMode::FileList;
        app.selected_file = 1;
        app.toggle_mark();
        assert!(app.clusters[0].files[1].marked_for_deletion);
    }

    #[test]
    fn toggle_mark_desmarca_si_ya_estaba_marcado() {
        let mut app = make_app(&[3]);
        app.clusters[0].files[0].marked_for_deletion = true;
        app.toggle_mark();
        assert!(!app.clusters[0].files[0].marked_for_deletion);
    }

    #[test]
    fn toggle_mark_no_permite_marcar_ultimo_archivo() {
        let mut app = make_app(&[2]);
        app.clusters[0].files[0].marked_for_deletion = true;
        app.selected_file = 1;
        app.toggle_mark();
        assert!(
            !app.clusters[0].files[1].marked_for_deletion,
            "No se debe poder marcar el ultimo archivo no marcado del cluster"
        );
    }

    #[test]
    fn execute_deletion_elimina_marcados_y_reduce_cluster() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut app = App::new(
            vec![ImageCluster {
                root_hash: 0,
                files: vec![
                    ClusterFile {
                        path: path.clone(),
                        distance: 0,
                        size_bytes: 1024,
                        marked_for_deletion: true,
                    },
                    ClusterFile {
                        path: PathBuf::from("/tmp/keep1.jpg"),
                        distance: 0,
                        size_bytes: 2048,
                        marked_for_deletion: false,
                    },
                    ClusterFile {
                        path: PathBuf::from("/tmp/keep2.jpg"),
                        distance: 0,
                        size_bytes: 3072,
                        marked_for_deletion: false,
                    },
                ],
            }],
            false,
        );

        let initial_len = app.clusters[0].files.len();
        let (deleted, errors) = app.execute_deletion();

        assert_eq!(deleted, 1);
        assert!(errors.is_empty());
        assert_eq!(app.clusters[0].files.len(), initial_len - 1);
    }

    #[test]
    fn navegar_clusters_actualiza_selected_file_a_cero() {
        let mut app = make_app(&[3, 4]);
        app.selected_cluster = 0;
        app.selected_file = 2;
        app.selected_cluster = 1;
        app.selected_file = 0;
        assert_eq!(app.selected_file, 0);
    }
}
