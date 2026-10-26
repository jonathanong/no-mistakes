use super::*;

impl Taint {
    pub fn prefix(mut self, key: &str) -> Self {
        if !self.broad {
            for paths in [&mut self.paths, &mut self.open] {
                *paths = std::mem::take(paths)
                    .into_iter()
                    .map(|mut path| {
                        path.insert(0, key.into());
                        path
                    })
                    .collect();
            }
            self.producers = self
                .producers
                .into_iter()
                .map(|(mut path, id)| {
                    path.insert(0, key.into());
                    (path, id)
                })
                .collect();
            self.objects = self
                .objects
                .into_iter()
                .map(|(mut path, at)| {
                    path.insert(0, key.into());
                    (path, at)
                })
                .collect();
            if self
                .paths
                .iter()
                .chain(&self.open)
                .chain(self.producers.iter().map(|(path, _)| path))
                .any(|path| path.len() > PATH_LIMIT)
            {
                self.widen();
            }
        }
        self
    }
    pub fn read(mut self, key: Option<&str>) -> Self {
        if self.broad {
            return self;
        }
        let Some(key) = key else {
            if self.present() || !self.open.is_empty() {
                self.widen();
            }
            return self;
        };
        let open = self.latent();
        self.producers = self
            .producers
            .into_iter()
            .filter_map(|(mut path, id)| {
                if path.first().is_some_and(|part| part == key) {
                    path.remove(0);
                    Some((path, id))
                } else {
                    None
                }
            })
            .collect();
        self.objects = self
            .objects
            .into_iter()
            .filter_map(|(mut path, at)| {
                if path.first().is_some_and(|part| part == key) {
                    path.remove(0);
                    Some((path, at))
                } else {
                    None
                }
            })
            .collect();
        self.paths = self
            .paths
            .into_iter()
            .filter_map(|mut path| {
                if path.first().is_some_and(|part| part == key) {
                    path.remove(0);
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        self.open = self
            .open
            .into_iter()
            .filter_map(|mut path| {
                if path.is_empty() {
                    Some(path)
                } else if path[0] == key {
                    path.remove(0);
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        if open && matches!(key, "query" | "call" | "apply" | "bind") {
            self.paths.insert(Vec::new());
        }
        self
    }
    pub fn spread(mut self) -> Self {
        self.paths.remove(&Vec::new());
        self.producers.retain(|(path, _)| !path.is_empty());
        self.objects.retain(|(path, _)| !path.is_empty());
        self
    }
}
