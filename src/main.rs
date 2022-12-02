use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};
use camera::Anchor;
use utils::create_mesh;
use voxels::{chunk::chunk_manager::ChunkManager, data::voxel_octree::VoxelMode, utils::key_to_world_coord_f32};

mod camera;
mod utils;
mod physics;

fn main() {
  App::new()
    // .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .add_plugin(camera::CustomPlugin)
    .add_plugin(physics::CustomPlugin)
    .add_startup_system(setup)
    .add_startup_system(add_player)
    .run();
}

/// set up a simple 3D scene
fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let chunk_manager = ChunkManager::default();
  let mut config = chunk_manager.config;

  let key = [0, -1, 0];
  let chunk = ChunkManager::new_chunk(
    &key, 
    config.depth, 
    config.depth, 
    config.noise
  );
  let data = chunk.octree.compute_mesh2(VoxelMode::SurfaceNets, &mut config.voxel_reuse);
  if data.indices.len() == 0 {
    print!("Return");
    return;
  }
  let mesh = create_mesh(
    &mut meshes, 
    data.positions, 
    data.normals, 
    data.uvs, 
    data.indices
  );

  let coord_f32 = key_to_world_coord_f32(&key, config.seamless_size);

  commands
    .spawn_bundle(PbrBundle {
      mesh: mesh,
      material: materials.add(Color::rgba(0.5, 0.5, 0.5, 0.3).into()),
      transform: Transform::from_xyz(coord_f32[0], coord_f32[1], coord_f32[2]),
      ..Default::default()
    });

  // light
  commands.spawn_bundle(PointLightBundle {
    point_light: PointLight {
      intensity: 1500.0,
      shadows_enabled: true,
      ..default()
    },
    transform: Transform::from_xyz(4.0, 8.0, 4.0),
    ..default()
  });

  // plane
  commands.spawn_bundle(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
    material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
    ..default()
  });
}


fn add_player(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let depth = 1.0;
  let radius = 1.0;
  commands
    .spawn()
    .insert(Transform::from_translation(
      Vec3::new(0.0, 3.0, -5.0)
    ))
    .insert(GlobalTransform::default())
    .insert(Anchor::default())
    .with_children(|parent| {
      parent.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
          depth: depth,
          radius: radius,
          ..Default::default()
        })),
        material: materials.add(StandardMaterial {
          base_color: Color::rgba(0.8, 0.7, 0.6, 0.5).into(),
          alpha_mode: AlphaMode::Blend,
          ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
      });
    });
}


/*
  Goal
    Make iteration faster

  Implementation
    Create part by part deployment in this separate repo
    Get the terrain editing to work
    Makeshift the features for now
    Make it work
    Then solve how to bridge the difference between the prototype repo to the production repo

    Current
      Make terrain editing work
      Player movement
      Raycast

*/