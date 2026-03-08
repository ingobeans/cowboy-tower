use quad_storage::LocalStorage;

pub struct SaveManager {
    storage: LocalStorage,
    pub level: usize,
}

impl SaveManager {
    pub fn new() -> Self {
        let storage = LocalStorage::default();
        let level = storage
            .get("level")
            .and_then(|f| f.parse().ok())
            .unwrap_or(0);
        Self { storage, level }
    }
    pub fn save(&mut self, level: usize) {
        self.storage.set("level", &level.to_string());
    }
}
