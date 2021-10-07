use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*, tasks::{AsyncComputeTaskPool, ComputeTaskPool, TaskPoolBuilder}};
use bevy_rapier3d::{physics::{ColliderBundle, RapierConfiguration, RapierPhysicsPlugin}, prelude::{ColliderMassProps, ColliderPosition, ColliderShape, ColliderType}};

use bevy_rapier3d::prelude::*;

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

  for index in 0..20000 {
    let size = 1.0;
    let collider = ColliderBundle {
      position: ColliderPosition(Vec3::new(index as f32 * size, 0.0, 0.0).into()),
      shape: ColliderShape::cuboid(size, size, size),
      flags: ColliderFlags {
        active_collision_types: ActiveCollisionTypes::DYNAMIC_STATIC,
        ..ColliderFlags::default()
      },
      ..ColliderBundle::default()
    };

    commands
      .spawn()
      .insert_bundle(collider)
      .insert(ColliderPositionSync::Discrete);
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

