use bevy::prelude::*;
use crate::utils::Math;

use super::{Anchor, CameraSettings};

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system_to_stage(CoreStage::PostUpdate, rotate);
  }
}

fn rotate(
  anchors: Query<&Anchor>,
  mut cam: Query<(&mut Transform, &CameraSettings)>
) {
  let mut target = Vec3::ZERO;
  for a in anchors.iter() {
    target = a.pos.clone();
  }

  for (mut trans, settings) in cam.iter_mut() {
    let yaw_radians = settings.yaw.to_radians();
    let pitch_radians = settings.pitch.to_radians();

    let cam_look_at = Math::rot_to_look_at(Vec3::new(pitch_radians, yaw_radians, 0.0)) ;
    let cam_pos = target + (cam_look_at * 10.0);
    
    let new = Transform::from_xyz(cam_pos[0], cam_pos[1], cam_pos[2])
      .looking_at(target, Vec3::Y);

    trans.translation = new.translation;
    trans.rotation = new.rotation;
    trans.scale = new.scale;
  }
}