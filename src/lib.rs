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
    path::Path,
    marker::PhantomData
};
use ron::de::from_bytes;
use thiserror::Error;


#[derive(Resource)]
// This struct hold the handle to the asset so it doesn't disappear
// It also transfers the path term from the new function on the plugin
// to the function where the asset is loaded
pub struct ConfigFileHolder<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    path: &'static str,
    handle: Option<Handle<A>>,
    _marker: PhantomData<A>
}


// The plugin that is instansiated in the .add_plugins()
// Apparently (which is maybe obvious), you can implement fields in your plugins,
// which means that when you call ::new() you can enter data such as the path
pub struct EasyConfigPlugin<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    path: &'static str,
    _marker: PhantomData<A>
}


impl<A> EasyConfigPlugin<A> 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    pub fn new(path: &'static str) -> Self {
        Self {
            path: path,
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
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    fn build(&self, app: &mut App) {
        app
            .init_asset::<A>()
            .register_asset_loader(ConfigFileAssetLoader::<A> {
                _marker: PhantomData
            })
            .insert_resource(ConfigFileHolder::<A> {
                path: self.path,
                handle: None,
                _marker: PhantomData
            })
            // .init_resource::<A>()
            .add_systems(PreStartup, add_asset_to_config_file_holder::<A>)
            .add_systems(Update, update_resource::<A>);
    }
}


// Gets the handle and stores it in ConfigFileHolder
fn add_asset_to_config_file_holder<A>(
    mut config_file_holder: ResMut<ConfigFileHolder<A>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) 
where
    for<'de> A: serde::Deserialize<'de> + Asset + Resource + Clone
{
    let base_path = bevy::asset::io::file::FileAssetReader::get_base_path();
    let assets_path = base_path.join(Path::new("assets"));
    let config_path = assets_path.join(Path::new(config_file_holder.path));

    let config_string = std::fs::read_to_string(config_path.as_path()).unwrap();
    let config_file: A = ron::from_str(&config_string).unwrap();

    let config_file_handle: Handle<A> = asset_server.load(config_file_holder.path);
    config_file_holder.handle = Some(config_file_handle.clone());

    commands.insert_resource(config_file);
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
        println!("{:#?}", ev);
        match ev {
            AssetEvent::Modified { id } => {
                let config_file = config_files.get(id.clone());

                println!("ahg");

                match config_file {
                    Some(config_file) => commands.insert_resource(config_file.clone()),
                    None => println!("Oh nosies 2")
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