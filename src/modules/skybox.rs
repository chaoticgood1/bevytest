use crate::bevy_skybox;
use crate::bevy_skybox::lib;
use crate::camera;
use bevy::render::camera::PerspectiveProjection;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use image::load_from_memory;
use image::DynamicImage;
use image::{open, Rgb, RgbImage};
use rand::Rng;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_resource(AssetLoader {
        list: std::collections::HashMap::new(),
      })
      .add_startup_system(load_assets.system())
      .add_startup_system(setup.system())
      .add_system(loading.system())
      .add_plugin(camera::camera::CustomPlugin)
      .add_plugin(lib::SkyboxPlugin::from_image_file("sky1.png"));
  }
}

fn setup(
  commands: &mut Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
) {
  let cam = Camera3dBundle {
    transform: Transform::from_matrix(Mat4::from_translation(Vec3::new(0.0, 2.0, -4.0)))
      .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    perspective_projection: PerspectiveProjection {
      far: 200.0,
      ..Default::default()
    },
    ..Default::default()
  };

  commands
    .spawn(cam)
    // .with(FlyCamera::default())
    .with(camera::fly_camera::FlyCamera::default())
    .with(lib::SkyboxCamera)
    .with_children(|parent| {
      // Add a light source for the board that moves with the camera.
      parent.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 30.0, 0.0)),
        ..Default::default()
      });
    });

  // Add a static "board" as some foreground to show camera movement.
  let mut rng = rand::thread_rng();
  for i in -20..=20 {
    for j in -20..=20 {
      // Each square is a random shade of green.
      let br = rng.gen::<f32>() * 0.4 + 0.6;
      let col = Color::rgb(0.6 * br, 1. * br, 0.6 * br);
      commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(col.into()),
        transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
        ..Default::default()
      });
    }
  }
}

struct AssetLoader {
  list: std::collections::HashMap<String, bool>,
}

fn load_assets(mut assets: ResMut<AssetLoader>, mut asset_server: ResMut<AssetServer>) {
  assets.list.insert("sky1.png".to_string(), false);

  for (key, val) in assets.list.iter() {
    // asset_server.load(key.as_str());
    asset_server.load_untyped(key.as_str());
  }
}

fn loading(
  commands: &mut Commands,
  mut assets: ResMut<AssetLoader>,
  mut asset_server: ResMut<AssetServer>,
  textures: ResMut<Assets<Texture>>,

  mut materials: ResMut<Assets<StandardMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  for (key, loaded) in assets.list.iter_mut() {
    if asset_server.get_load_state(key.as_str()) == LoadState::Loaded {
      // info!("LoadState::Loaded");
      if !*loaded {
        *loaded = true;
        info!("Loaded {:?}", key);
        bevy_skybox::image::create_skybox_new(
          commands,
          &mut materials,
          &mut meshes,
          &mut asset_server,
          &textures,
          key.as_str(),
        );
      }
    }
  }
}
