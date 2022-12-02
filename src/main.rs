use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};
use camera::Anchor;
use voxels::{chunk::chunk_manager::ChunkManager, data::voxel_octree::VoxelMode, utils::key_to_world_coord_f32};

mod camera;
mod utils;

fn main() {
  App::new()
    // .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .add_plugin(camera::CustomPlugin)
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
  // plane
  commands.spawn_bundle(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
    material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    ..default()
  });

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

  // // camera
  // commands.spawn_bundle(PerspectiveCameraBundle {
  //   transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
  //   ..default()
  // });
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



pub fn create_mesh(
  meshes: &mut ResMut<Assets<Mesh>>,
  positions: Vec<[f32; 3]>,
  normals: Vec<[f32; 3]>,
  uvs: Vec<[f32; 2]>,
  indices: Vec<u32>,
) -> Handle<Mesh> {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
  mesh.set_indices(Some(Indices::U32(indices)));
  meshes.add(mesh)
}