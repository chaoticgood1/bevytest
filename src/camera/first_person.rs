use bevy::prelude::*;

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

    trans.rotation =
      Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(-Vec3::X, pitch_radians);

    trans.translation = target.clone();
  }
}