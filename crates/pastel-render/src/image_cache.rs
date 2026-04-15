use pastel_lang::ir::IrAsset;
use skia_safe::{Data, Image};
use std::collections::HashMap;

#[derive(Default)]
pub struct ImageCache {
    cache: HashMap<String, Option<Image>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn load_from_assets(&mut self, assets: &[IrAsset]) {
        for asset in assets {
            if asset.kind != "image" {
                continue;
            }
            let img = asset
                .resolved_path
                .as_ref()
                .and_then(|p| {
                    eprintln!("[image_cache] Loading: {} -> {}", asset.id, p.display());
                    std::fs::read(p).ok()
                })
                .and_then(|bytes| {
                    eprintln!("[image_cache]   {} bytes read", bytes.len());
                    let data = Data::new_copy(&bytes);
                    let result = Image::from_encoded(data);
                    eprintln!(
                        "[image_cache]   decode: {}",
                        if result.is_some() { "OK" } else { "FAILED" }
                    );
                    result
                });
            if asset.resolved_path.is_none() {
                eprintln!("[image_cache] No resolved_path for: {}", asset.id);
            }
            self.cache.insert(asset.id.clone(), img);
        }
    }

    pub fn get(&self, asset_id: &str) -> Option<&Image> {
        self.cache.get(asset_id)?.as_ref()
    }
}
