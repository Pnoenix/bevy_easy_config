use bevy::{
    asset::{Asset, Assets, AssetApp, AssetEvent, AssetLoader, AsyncReadExt, AssetServer, LoadContext, io::Reader, Handle},
    app::{PreStartup, Update, App, Plugin},
    ecs::{
        system::{Commands, Resource},
        change_detection::{Res, ResMut},
        event::EventReader
    },
};
use std::{
    path::PathBuf,
    marker::PhantomData
};
use ron::de::from_bytes;
use thiserror::Error;


#[derive(Resource)]
// This struct hold the handle to the asset so it doesn't disappear
struct ConfigFileHandle<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    handle: Option<Handle<A>>,
    _marker: PhantomData<A>
}


#[derive(Resource)]
struct ConfigFileSettings<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    path: PathBuf,
    _marker: PhantomData<A>
}


// The plugin that is instansiated in the .add_plugins()
// Apparently (which is maybe obvious), you can implement fields in your plugins,
// which means that when you call ::new() you can enter data such as the path
pub struct EasyConfigPlugin<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    path: PathBuf,
    _marker: PhantomData<A>
}


impl<A> EasyConfigPlugin<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _marker: PhantomData
        }
    }
}


// Since you call with ::new() the &self argument in the build function
// can be used to extract the path using self.path
// self.path is passed into ConfigFileHolder and instansiated as a resource
// Which can be used from anywhere to get the path 
impl<A> Plugin for EasyConfigPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone,
{
    fn build(&self, app: &mut App) {
        app
            .init_asset::<A>()
            .register_asset_loader(ConfigFileAssetLoader::<A> {
                _marker: PhantomData
            })
            .add_systems(Update, update_resource::<A>)
            .insert_resource(ConfigFileHandle::<A> {
                handle: None,
                _marker: PhantomData
            })
            .insert_resource(ConfigFileSettings::<A> {
                path: self.path.clone(),
                _marker: PhantomData
            })
            .add_systems(PreStartup, load_config_file::<A>);            
    }
}


// Gets the handle and stores it in ConfigFileHolder
fn load_config_file<A>(
    mut config_file_handle: ResMut<ConfigFileHandle<A>>,
    mut commands: Commands,
    config_file_settings: Res<ConfigFileSettings<A>>,
    asset_server: Res<AssetServer>,
) 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    let mut path = bevy::asset::io::file::FileAssetReader::get_base_path();
    path.push("assets");
    path.push(config_file_settings.path.as_path());

    let config_string = std::fs::read_to_string(path.as_path()).unwrap();
    let config_file: A = ron::from_str(&config_string).unwrap();

    commands.insert_resource(config_file);
    
    let handle: Handle<A> = asset_server.load(path.clone());
    config_file_handle.handle = Some(handle.clone());
}


// When the asset is finished loading, it is extracted and inserted as a resource.
// If a modification happens (hot reloading / saving the file the asset references),
// it replaces the current resource with the new modified asset
fn update_resource<A>(
    mut ev_asset: EventReader<AssetEvent<A>>,
    config_files: Res<Assets<A>>,
    mut commands: Commands
) 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Modified { id } => {
                let config_file = config_files.get(id.clone());

                match config_file {
                    Some(config_file) => commands.insert_resource(config_file.clone()),
                    None => {}
                }
            },
            _ => {}
        }
    }
}


// All of this below is magic, but the magic works so no reason to fix lol
// Was I 'inspired' by bevy_common_assets... maybe :D (Thank you)
pub struct ConfigFileAssetLoader<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    _marker: PhantomData<A>,
}


/// Possible errors that can be produced by [`ConfigFileAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigFileLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON Error](ron::error::SpannedError)
    #[error("Could not parse RON: {0}")]
    RonError(#[from] ron::error::SpannedError),
}


impl<A> AssetLoader for ConfigFileAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    type Asset = A;
    type Settings = ();
    type Error = ConfigFileLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = from_bytes::<A>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}