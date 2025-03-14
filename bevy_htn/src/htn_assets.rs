use crate::dsl::parse_htn;
use crate::htn::HTN;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use futures_lite::AsyncReadExt;
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Default)]
struct HtnAssetLoader<T: Reflect + TypePath + Default> {
    _phantom: PhantomData<T>,
}

impl<T: Reflect + TypePath + Default> AssetLoader for HtnAssetLoader<T> {
    type Asset = HtnAsset<T>;
    type Settings = ();
    type Error = HtnAssetError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut value = String::new();
        reader.read_to_string(&mut value).await?;
        // info!("Loaded htn: {}", value);
        Ok(HtnAsset {
            htn: parse_htn::<T>(&value),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["htn"]
    }
}

#[derive(Asset, TypePath)]
pub struct HtnAsset<T: Reflect + TypePath> {
    pub htn: HTN<T>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HtnAssetError {
    /// An [IO](std::io) Error
    #[error("Could not load htn: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct HtnAssetPlugin<T: Reflect + TypePath + Default> {
    _phantom: PhantomData<T>,
}

impl<T: Reflect + TypePath + Default> Plugin for HtnAssetPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<HtnAssetLoader<T>>();
        app.init_asset::<HtnAsset<T>>();
    }
}
