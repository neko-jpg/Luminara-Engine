use crate::{Asset, AssetId};
use luminara_core::shared_types::Resource;
use std::collections::HashMap;

pub struct AssetStorage<T: Asset> {
    assets: HashMap<AssetId, T>,
}

impl<T: Asset> AssetStorage<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: AssetId, asset: T) {
        self.assets.insert(id, asset);
    }

    pub fn get(&self, id: AssetId) -> Option<&T> {
        self.assets.get(&id)
    }

    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut T> {
        self.assets.get_mut(&id)
    }

    pub fn remove(&mut self, id: AssetId) -> Option<T> {
        self.assets.remove(&id)
    }

    pub fn contains(&self, id: AssetId) -> bool {
        self.assets.contains_key(&id)
    }
}

impl<T: Asset> Default for AssetStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Asset> Resource for AssetStorage<T> {}
