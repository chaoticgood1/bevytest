use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, ComputeTaskPool, TaskPoolBuilder}};
use bevy_rapier3d::{physics::ColliderBundle, prelude::{ColliderMassProps, ColliderPosition, ColliderShape, ColliderType}};

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
  let total_cpu = bevy::tasks::logical_core_count();
  let compute_cpu = (total_cpu as f32 * 0.75) as usize;
  let async_cpu = total_cpu - compute_cpu;

  println!("total_cpu {}, compute_cpu {} async_cpu {}", total_cpu, compute_cpu, async_cpu);


  App::build()
    .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .insert_resource(AsyncComputeTaskPool(
      TaskPoolBuilder::default()
        .num_threads(async_cpu)
        .thread_name("Async Compute Task Pool".to_string())
        .build(),
    ))
    .insert_resource(ComputeTaskPool(
      TaskPoolBuilder::default()
        .num_threads(compute_cpu)
        .thread_name("Compute Task Pool".to_string())
        .build(),
    ))
    .add_startup_system(startup.system())
    // .add_system(system.system())
    .add_system(fps_system.system())
    .run();
}

fn startup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  // compute: Res<TaskPoolBuilder>
) {
  println!("core count {:?}", bevy::tasks::logical_core_count());
  // println!("bevy {:?}", compute.);

  // light
  commands.spawn_bundle(LightBundle {
    transform: Transform::from_xyz(4.0, 8.0, 4.0),
    ..Default::default()
  });
  // camera
  commands.spawn_bundle(PerspectiveCameraBundle {
      transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
      ..Default::default()
  });

  for index in 0..3000 {
    let size = 1.0;
    let collider = ColliderBundle {
      collider_type: ColliderType::Sensor,
      mass_properties: ColliderMassProps::Density(0.0),
      position: ColliderPosition(Vec3::new(index as f32, 0.0, 0.0).into()),
      shape: ColliderShape::cuboid(size, size, size),
      ..ColliderBundle::default()
    };

    commands
      .spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
      })
      .insert_bundle(collider);
  }
  
}

fn system(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut local_res: Local<LocalResource>,
  time: Res<Time>,
) {
  if local_res.collider_count >= local_res.limit {
    return;
  }

  local_res.duration += time.delta_seconds();
  
  if local_res.duration >= local_res.interval {
    local_res.interval -= local_res.duration;
    local_res.collider_count += 1;

    let size = 1.0;
    let collider = ColliderBundle {
      collider_type: ColliderType::Sensor,
      mass_properties: ColliderMassProps::Density(0.0),
      position: ColliderPosition(Vec3::new(local_res.collider_count as f32, 0.0, 0.0).into()),
      shape: ColliderShape::cuboid(size, size, size),
      ..ColliderBundle::default()
    };

    commands
      .spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
      })
      .insert_bundle(collider)
      ;

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