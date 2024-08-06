[![crates.io](https://img.shields.io/crates/v/bevy_easy_config.svg)](https://crates.io/crates/bevy_easy_config)
[![license](https://img.shields.io/crates/l/bevy_easy_config)](https://github.com/Pnoenix/bevy_easy_config#license)


# Bevy Easy Config
Bevy Easy Config is a plugin that allows you to load config files easily and instantiate them as a resource.

## Usage
First define the struct that you would like to load, and impl/derive the relevant traits:
```rust
// Define the struct to load
#[derive(Deserialize, Asset, Resource, Clone, TypePath)]
struct Settings {
    some_keybind: KeyCode
}

// Default also needs to be implemented
impl Default for Settings {
    fn default() -> Self {
        Self {
            some_keybind: KeyCode::KeyW
        }
    }
}

```
The add it to your app:
```rust
use bevy_easy_config::EasyConfigPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EasyConfigPlugin::<Settings>::new("settings.ron"),
        ))
        .run();
}
```
This will load the file located at `assets/settings.ron`, into the Settings struct and insert Settings as a resource.

```rust
fn some_random_function(
    settings: Res<Settings>
) {
    // ... Your awesome code here
}
```

## Notices
Until the asset is loaded, when accessing the Settings resource, you will get the values from `Settings::default()`.
As far as I know there isn't really a good way around this, so you will have to work with the default values for the
first few ticks, until the asset is loaded. When the asset is loaded, it will automatically replace the default values.

## Compatible Bevy versions
| `bevy_common_assets`  | `bevy` |
|:----------------------|:-------|
| `0.1.0`               | `0.14` |

## License
Dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

## Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
