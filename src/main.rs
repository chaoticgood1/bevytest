pub mod bevy_mod_picking_test;

use bevy::prelude::*;

fn main() {
  App::build()
    .add_resource(Msaa { samples: 4 })
    .add_plugin(bevy_mod_picking_test::CustomPlugin)
    .run()
}