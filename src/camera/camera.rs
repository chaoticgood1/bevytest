use crate::camera::fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::prelude::*;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FlyCameraPlugin)
            .add_startup_system(startup.system())
            .add_system(system.system()); // For debugging
    }
}

fn startup(commands: &mut Commands) {
    let mut cam_trans = Transform::from_translation(Vec3::new(0.0, 3.0, -10.0));
    cam_trans.look_at(Vec3::new(0.0, 0.0, 1.0), Vec3::unit_y());
    commands
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 20.0, 4.0)),
            ..Default::default()
        })
        .spawn(Camera3dBundle {
            transform: cam_trans, // Temporary, refactor later
            ..Default::default()
        })
        .with(FlyCamera::default());
}

fn system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    // keyboard_input: Local<Input<KeyCode>>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    // Detect game state
    for (mut options, mut _transform) in query.iter_mut() {
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
