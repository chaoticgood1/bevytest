pub mod third_person;
pub mod first_person;

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};

use crate::utils::Math;

/*
  Third person camera
*/


pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(PointerState::default())
      .add_startup_system(add_cam)
      .add_startup_system(add_light);

    app
      .add_system(mouse_motion)
      .add_system_to_stage(CoreStage::PreUpdate, anchor_rotation)
      .add_system_to_stage(CoreStage::PostUpdate, movement)
      ;

    app
      .add_plugin(third_person::CustomPlugin)
      // .add_plugin(first_person::CustomPlugin)
      ;
  }
}

/* Setup */
fn add_cam(mut commands: Commands) {
  commands
    .spawn_bundle(PerspectiveCameraBundle {
      transform: Transform::from_xyz(0.0, 0.5, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
      ..default()
    })
    .insert(CameraSettings::default());
}

fn add_light(mut commands: Commands) {
  const HALF_SIZE: f32 = 10.0;
  commands.spawn_bundle(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadow_projection: OrthographicProjection {
        left: -HALF_SIZE,
        right: HALF_SIZE,
        bottom: -HALF_SIZE,
        top: HALF_SIZE,
        near: -10.0 * HALF_SIZE,
        far: 10.0 * HALF_SIZE,
        ..default()
      },
      shadows_enabled: true,
      illuminance: 10000.0,
      ..default()
    },
    transform: Transform {
      translation: Vec3::new(0.0, 100.0, 0.0),
      rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
      ..default()
    },
    ..default()
  });
}
/* Setup */

/* Setting Mouse settings, have to change name later */
fn mouse_motion(
  time: Res<Time>,
  mut state: ResMut<PointerState>,
  mut mouse_motion_events: EventReader<MouseMotion>,
  mut ev_mousebtn: EventReader<MouseButtonInput>,
  mut ev_cursor: EventReader<CursorMoved>,

  mut cam_settings: Query<&mut CameraSettings>
) {
  let mut delta: Vec2 = Vec2::ZERO;
  for event in mouse_motion_events.iter() {
    delta += event.delta;
  }

  for ev_mouse in ev_mousebtn.iter() {
    if ev_mouse.state.is_pressed() && ev_mouse.button == MouseButton::Left {
      state.dragged = true;
    }

    if !ev_mouse.state.is_pressed() && ev_mouse.button == MouseButton::Left {
      state.dragged = false;
    }
  }

  for ev in ev_cursor.iter() {
    if state.last_cursor_pos.length_squared() < 0.1 || !state.dragged {
      state.last_cursor_pos = ev.position;
      return;
    }
    delta = ev.position - state.last_cursor_pos;
    // delta.y *= -1.0;
    delta.x *= -1.0;
    state.last_cursor_pos = ev.position;


    for mut settings in cam_settings.iter_mut() {
      settings.pitch -= delta.y * settings.pitch_speed * time.delta_seconds();
      settings.yaw += delta.x * settings.yaw_speed * time.delta_seconds();
      
      settings.pitch = settings.pitch.clamp(-89.9, 89.9);
  
      // info!("yaw {} {}", settings.yaw, settings.pitch);
    }
  }
}

fn anchor_rotation(
  mut anchors: Query<&mut Anchor>,
  cam: Query<&CameraSettings>
) {
  for mut a in anchors.iter_mut() {
    // target = a.0.clone();
    for settings in cam.iter() {
      let yaw_radians = settings.yaw.to_radians();
      let pitch_radians = settings.pitch.to_radians();
  
      let cam_look_at = Math::rot_to_look_at(Vec3::new(pitch_radians, yaw_radians, 0.0));
      a.dir = cam_look_at * -1.0;
    }
  }
}

fn movement(
  mut anchors: Query<(&Transform, &mut Anchor)>
) {
  for (trans, mut anchor) in anchors.iter_mut() {
    anchor.pos = trans.translation.clone();
  }
}



#[derive(Component)]
pub struct CameraSettings {
  pub pitch: f32,
  pub yaw: f32,
  pub pitch_speed: f32,
  pub yaw_speed: f32,
}

impl Default for CameraSettings {
  fn default() -> Self {
    Self {
      pitch: 0.0,
      yaw: 180.0,
      pitch_speed: 10.0,
      yaw_speed: 10.0
    }
  }
}

#[derive(Component)]
struct PointerState {
  dragged: bool,
  last_cursor_pos: Vec2,
}

impl Default for PointerState {
  fn default() -> Self {
    Self {
      dragged: false,
      last_cursor_pos: Vec2::ZERO,
    }
  }
}

#[derive(Component, Default)]
pub struct Anchor {
  pub pos: Vec3,
  pub dir: Vec3,
}