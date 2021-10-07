use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*, tasks::{AsyncComputeTaskPool, ComputeTaskPool, TaskPoolBuilder}};
use bevy_rapier3d::{physics::{ColliderBundle, RapierConfiguration, RapierPhysicsPlugin}, prelude::{ColliderMassProps, ColliderPosition, ColliderShape, ColliderType}};

use bevy_rapier3d::prelude::*;
use bevy::render::mesh;

struct LocalResource {
  pub collider_count: u32,
  pub limit: u32,
  pub duration: f32,
  pub interval: f32,
}

impl Default for LocalResource {
  fn default() -> Self {
    LocalResource {
      collider_count: 0,
      limit: 1500,
      duration: 0.0,
      interval: 0.00,
    }
  }
}

struct FrameResource {
  pub duration: f32,
  pub interval: f32,
  pub count: f32,
  pub current_count: f32,
  pub avg_count: f32,
}

impl Default for FrameResource {
  fn default() -> Self {
    FrameResource {
      duration: 0.0,
      interval: 1.0,
      count: 0.0,
      current_count: 0.0,
      avg_count: 0.0,
    }
  }
}

fn main() {
  // let total_cpu = bevy::tasks::logical_core_count();
  // let compute_cpu = (total_cpu as f32 * 0.75) as usize;
  // let async_cpu = total_cpu - compute_cpu;

  // println!("total_cpu {}, compute_cpu {} async_cpu {}", total_cpu, compute_cpu, async_cpu);

  App::build()
    .add_plugins(DefaultPlugins)
    // .add_plugins(MinimalPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(LogDiagnosticsPlugin::default())
    // .insert_resource(AsyncComputeTaskPool(
    //   TaskPoolBuilder::default()
    //     .num_threads(async_cpu)
    //     .thread_name("Async Compute Task Pool".to_string())
    //     .build(),
    // ))
    // .insert_resource(ComputeTaskPool(
    //   TaskPoolBuilder::default()
    //     .num_threads(compute_cpu)
    //     .thread_name("Compute Task Pool".to_string())
    //     .build(),
    // ))
    .add_startup_system(startup.system())
    // .add_system(system.system())
    .add_system(fps_system.system())
    .run();
}

fn startup(
  mut commands: Commands,
  // mut meshes: ResMut<Assets<Mesh>>,
  // mut materials: ResMut<Assets<StandardMaterial>>,
  mut rapier_config: ResMut<RapierConfiguration>,
) {
  // rapier_config.query_pipeline_active = false;
  // rapier_config.physics_pipeline_active = false;
  println!("core count {:?}", bevy::tasks::logical_core_count());
  println!("Testing");

  // // light
  // commands.spawn_bundle(LightBundle {
  //   transform: Transform::from_xyz(4.0, 8.0, 4.0),
  //   ..Default::default()
  // });
  // // camera
  // commands.spawn_bundle(PerspectiveCameraBundle {
  //     transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
  //     ..Default::default()
  // });

  let positions = vec![[1.5, 1.5, 4.5], [1.5, 2.5, 4.5], [1.5, 3.5, 4.5], [1.5, 4.5, 4.5], [1.5, 5.5, 1.5], [1.5, 5.5, 2.5], [1.5, 5.5, 3.5], [1.5, 5.25, 4.25], [2.5, 1.5, 4.5], [2.5, 2.5, 4.5], [2.5, 3.5, 4.5], [2.5, 4.5, 4.5], [2.5, 5.5, 1.5], [2.5, 5.5, 2.5], [2.5, 5.5, 3.5], [2.5, 5.25, 4.25], [3.5, 1.5, 4.5], [3.5, 2.5, 4.5], [3.5, 3.5, 4.5], [3.5, 4.5, 4.5], [3.5, 5.5, 1.5], [3.5, 5.5, 2.5], [3.5, 5.5, 3.5], [3.5, 5.25, 4.25], [4.5, 1.5, 4.5], [4.5, 2.5, 4.5], [4.5, 3.5, 4.5], [4.5, 4.5, 4.5], [4.5, 5.5, 1.5], [4.5, 5.5, 2.5], [4.5, 5.5, 3.5], [4.5, 5.25, 4.25], [5.5, 1.5, 1.5], [5.5, 1.5, 2.5], [5.5, 1.5, 3.5], [5.25, 1.5, 4.25], [5.5, 2.5, 1.5], [5.5, 2.5, 2.5], [5.5, 2.5, 3.5], [5.25, 2.5, 4.25], [5.5, 3.5, 1.5], [5.5, 3.5, 2.5], [5.5, 3.5, 3.5], [5.25, 3.5, 4.25], [5.5, 4.5, 1.5], [5.5, 4.5, 2.5], [5.5, 4.5, 3.5], [5.25, 4.5, 4.25], [5.25, 5.25, 1.5], [5.25, 5.25, 2.5], [5.25, 5.25, 3.5], [5.1666665, 5.1666665, 4.1666665]];
  let indices = vec![9, 1, 8, 1, 0, 8, 10, 2, 9, 2, 1, 9, 11, 3, 10, 3, 2, 10, 13, 12, 5, 5, 12, 4, 14, 13, 6, 6, 13, 5, 15, 7, 11, 7, 3, 11, 15, 14, 7, 7, 14, 6, 17, 9, 16, 9, 8, 16, 18, 10, 17, 10, 9, 17, 19, 11, 18, 11, 10, 18, 21, 20, 13, 13, 20, 12, 22, 21, 14, 14, 21, 13, 23, 15, 19, 15, 11, 19, 23, 22, 15, 15, 22, 14, 25, 17, 24, 17, 16, 24, 26, 18, 25, 18, 17, 25, 27, 19, 26, 19, 18, 26, 29, 28, 21, 21, 28, 20, 30, 29, 22, 22, 29, 21, 31, 23, 27, 23, 19, 27, 31, 30, 23, 23, 30, 22, 37, 33, 36, 36, 33, 32, 38, 34, 37, 37, 34, 33, 39, 25, 35, 25, 24, 35, 39, 35, 38, 38, 35, 34, 41, 37, 40, 40, 37, 36, 42, 38, 41, 41, 38, 37, 43, 26, 39, 26, 25, 39, 43, 39, 42, 42, 39, 38, 45, 41, 44, 44, 41, 40, 46, 42, 45, 45, 42, 41, 47, 27, 43, 27, 26, 43, 47, 43, 46, 46, 43, 42, 49, 45, 48, 48, 45, 44, 49, 48, 29, 29, 48, 28, 50, 46, 49, 49, 46, 45, 50, 49, 30, 30, 49, 29, 51, 31, 47, 31, 27, 47, 51, 47, 50, 50, 47, 46, 51, 50, 31, 31, 50, 30];
  
  for index in 0..20000 {
    let shape = ColliderShape::trimesh(
      convert_arrayf32_to_points(positions.clone()),
      convert_flatu32_to_points(indices.clone()),
    );

    let size = 1.0;
    let collider = ColliderBundle {
      position: ColliderPosition(Vec3::new(index as f32 * size, 0.0, 0.0).into()),
      // shape: shape,
      shape: ColliderShape::cuboid(size, size, size),
      flags: ColliderFlags {
        active_collision_types: ActiveCollisionTypes::DYNAMIC_STATIC,
        ..ColliderFlags::default()
      },
      ..ColliderBundle::default()
    };

    commands
      // .spawn_bundle(PbrBundle {
      //   mesh: meshes.add(Mesh::from(mesh::shape::Cube { size: 1.0 })),
      //   material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
      //   transform: Transform::from_xyz(0.0, 0.5, 0.0),
      //   ..Default::default()
      // })
      .spawn()
      .insert_bundle(collider)
      .insert(ColliderPositionSync::Discrete);
  }
  
}

fn fps_system(
  collider_query: Query<&ColliderPosition>,
  mut frame: Local<FrameResource>,
  time: Res<Time>,
) {
  frame.duration += time.delta_seconds();
  frame.count += 1.0;

  if frame.duration >= frame.interval {
    frame.duration -= frame.interval;

    if frame.avg_count.is_infinite() || frame.avg_count <= 0.0 {
      frame.avg_count = frame.count;
    } else {
      frame.avg_count = (frame.avg_count + frame.count) * 0.5;
    }
    frame.current_count = frame.count;
    frame.count = 0.0;

    println!(
      "count {} fps avg_count {} collider_count {}",
      frame.current_count,
      frame.avg_count as i32,
      collider_query.iter().len()
    );
  }
}


pub fn convert_flatu32_to_points(array: Vec<u32>) -> Vec<[u32; 3]> {
  // let mut points: Vec<Point<u32>> = Vec::new();
  let mut points: Vec<[u32; 3]> = Vec::new();

  let slices: Vec<&[u32]> = array.chunks(3).collect();
  for slice in slices.iter() {
    // points.push(Point::new(slice[0], slice[1], slice[2]));
    points.push([slice[0], slice[1], slice[2]]);
  }
  points
}

pub fn convert_arrayf32_to_points(array: Vec<[f32; 3]>) -> Vec<Point<f32>> {
  let mut points: Vec<Point<f32>> = Vec::new();

  for array_p in array.iter() {
    // FIXME: Testing to move down the meshes to match the voxel
    // println!("pos {:?}", array_p[1]);
    points.push(Point::new(array_p[0], array_p[1], array_p[2]));
  }
  points
}

