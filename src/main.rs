pub mod bevy_mod_picking_test;
pub mod bevy_test;

use bevy::prelude::*;

fn main() {
  App::build()
    .add_resource(Msaa { samples: 4 })
    // .add_plugin(bevy_mod_picking_test::CustomPlugin)
    .add_plugin(bevy_test::CustomPlugin)
    .run()
}