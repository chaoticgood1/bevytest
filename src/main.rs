use bevy::prelude::*;
pub mod bevy_skybox;
pub mod camera;
pub mod modules;
use modules::skybox;

fn main() {
    let mut app = App::build();

    app
      .add_resource(Msaa { samples: 4 })
      .add_plugins(DefaultPlugins)
      .add_plugin(skybox::CustomPlugin)
      .add_plugin(bevy_webgl2::WebGL2Plugin)
      .run();
}
