use std::ops::Range;

use crate::camera::fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::prelude::*;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_plugin(FlyCameraPlugin)
      .add_startup_system(startup.system());

    app
      .insert_resource(LocalResource { pos: Vec3::ZERO })
      .add_system(bring_light.system())
      .add_system(system.system()); // For debugging
  }
}

#[derive(Clone)]
struct LocalResource {
  pub pos: Vec3,
}

fn startup(mut commands: Commands) {
  let mut cam_trans = Transform::from_translation(Vec3::new(0.0, 3.0, -10.0));
  cam_trans.look_at(Vec3::new(0.0, 0.0, 1.0), Vec3::Y);
  commands.spawn_bundle(LightBundle {
    light: Light {
      color: Color::WHITE,
      depth: Range {
        start: 0.0,
        end: 1000.0,
      },
      intensity: 10000.0,
      range: 1000.0,
      ..Default::default()
    },
    transform: Transform::from_translation(Vec3::new(4.0, 100.0, 4.0)),
    ..Default::default()
  });
  // .spawn(Camera3dBundle {
  commands
    .spawn_bundle(PerspectiveCameraBundle {
      transform: cam_trans, // Temporary, refactor later
      ..Default::default()
    })
    .insert(FlyCamera::default());

  // Work around implementation, not working
  // let window = web_sys::window().expect("no global `window` exists");
  // let document = window.document().expect("should have a document on window");
  // let body = document.body();
  // if let Some(bod) = body {
  //   bod.request_pointer_lock();
  //   info!("Executed");
  // }
}

fn bring_light(local: Res<LocalResource>, mut query: Query<(&Light, &mut Transform)>) {
  for (_, mut transform) in query.iter_mut() {
    transform.translation = local.pos;
  }
}

fn system(
  keyboard_input: Res<Input<KeyCode>>,
  // keyboard_input: Local<Input<KeyCode>>,
  mut local: ResMut<LocalResource>,
  mut query: Query<(&mut FlyCamera, &Transform)>,
) {
  // Detect game state
  for (mut options, mut _transform) in query.iter_mut() {
    local.pos = _transform.translation.clone();
    // options.enabled = false;
    // options.speed = 0.0;
    // info!("transform {:?}", _transform.translation);

    if keyboard_input.just_pressed(KeyCode::C) {
      // options.enabled = !options.enabled;
      // info!("Fly cam {}", options.enabled);
      if options.speed < 1.5 {
        options.speed = 1.5;
      } else {
        options.speed = 0.0;
      }
    }
  }
}
