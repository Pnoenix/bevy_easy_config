use bevy::prelude::*;
use bevy_easy_config::EasyConfigPlugin;
use serde::Deserialize;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Adds the settings struct as a resource, and loads it from "assets/settings.ron"
            EasyConfigPlugin::<Settings>::new("settings.ron")
        ))
        .add_systems(Startup, print_config_key)
        .add_systems(Update, print_on_keypress)
        .run();
}


/* The config struct must implement:
    - Deserialize, 
    - Asset,
    - Resource, 
    - Clone,
    - TypePath
    All traits except default can simply be implemented with #[derive()] 
*/
#[derive(Deserialize, Asset, Resource, Clone, TypePath)]
struct Settings {
    action_keybind: KeyCode
}


fn print_config_key(
    settings: Res<Settings>
) {
    println!("Settings loaded; current key: {:#?}", settings.action_keybind);
}


fn print_on_keypress(
    settings: Res<Settings>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Accessing the resource is just like if you have a normal resoruce,
    // except it also hot reloads whenever the file it is loaded from is saved. 
    if keyboard.just_pressed(settings.action_keybind) {
        println!("You pressed: {:#?}", settings.action_keybind)
    }
}