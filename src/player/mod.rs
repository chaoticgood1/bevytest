use bevy::{prelude::*, render::mesh::{Indices, VertexAttributeValues}};
use bevy_rapier3d::{na::Vector3, prelude::*};
use bevy::render::mesh;
use std::convert::{TryFrom, TryInto};
// use nalgebra::*;
use bevy_rapier3d::na;
use bevy_rapier3d::na::Point3;

struct Player;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .add_startup_system(ground.system())
      .add_startup_system(player.system())
      .add_system(update.system())
      .add_system(keyboard_input_system.system())
      ;
  }
}

fn keyboard_input_system(
  keyboard_input: Res<Input<KeyCode>>,
  mut player_query: Query<(&Player, &RigidBodyMassProps, &mut RigidBodyVelocity, &mut RigidBodyForces)>
  // mut player_query: Query<(&Player, &RigidBodyMassProps, &mut RigidBodyForces)>
) {
  if keyboard_input.just_pressed(KeyCode::W) {
    // for (_, rigid_mass, mut rigid_forces) in player_query.iter_mut() {
    //   println!("force");
    //   // rigid_forces.force += Vector3::new(0.0, 0.0, 1000000000.0);

    //   // rigid_forces.apply_force_at_point(rigid_mass, Vector3::new(0.0, 0.0, 13500.0), Point3::origin());
    // }

    for (_, rigid_mass, mut rigid_velocity, mut rigid_force) in player_query.iter_mut() {
      println!("force");
      // rigid_force.force = Vec3::new(0.0, 0.0, 100000.0).into();
      // rigid_force.torque = Vec3::new(0.0, 0.0, 100000.0).into();
      rigid_velocity.apply_impulse(rigid_mass, Vec3::new(100.0, 0.0, 0.0).into());
    }
  }
  if keyboard_input.just_pressed(KeyCode::S) {
  }

  if keyboard_input.just_released(KeyCode::A) {
  }

  if keyboard_input.just_released(KeyCode::D) {
  }
  
}

fn player(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // let mesh = Mesh::from(mesh::shape::Capsule {
  //   radius: 0.5,
  //   depth: 0.25,
  //   ..Default::default()
  // });

  let mesh = Mesh::from(mesh::shape::Cube {
    size: 0.5
  });

  let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);

  if let Some(VertexAttributeValues::Float3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    if let Some(Indices::U32(ind)) = mesh.indices() {
      let scale = Vec3::new(0.5, 0.5, 0.5);
      let coord = Vec3::new(0.0, 2.0, 0.0);

      // commands.spawn_bundle(PbrBundle {
      //   mesh: meshes.add(mesh.clone()),
      //   material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
      //   ..Default::default()
      // });
      let pbr = PbrBundle {
        mesh: meshes.add(mesh.clone()),
        material: materials.add(Color::rgb(0.3, 0.0, 0.3).into()),
        ..Default::default()
      };

      let locked_dofs = RigidBodyMassPropsFlags::ROTATION_LOCKED_Y
        | RigidBodyMassPropsFlags::ROTATION_LOCKED_Z;

      let rigid_body = RigidBodyBundle {
        position: [coord.x, coord.y, coord.z].into(),
        body_type: RigidBodyType::Dynamic,
        mass_properties: RigidBodyMassProps {
          // flags: locked_dofs,
          local_mprops: MassProperties::new(
            Point3::new(0.0, 0.0, 0.0), 1.0, Vector3::new(0.0, 0.0, 0.0)
          ),
          ..RigidBodyMassProps::default()
        },
        activation: RigidBodyActivation::cannot_sleep(),
        // ccd: RigidBodyCcd { ccd_enabled: true, ..Default::default() },
        ..RigidBodyBundle::default()
      };
      // println!("pos {:?}", pos);
      // println!("ind {:?}", ind);

      let collider = ColliderBundle {
        // shape: ColliderShape::cuboid(scale.x, scale.y, scale.z),
        shape: ColliderShape::trimesh(
          convert_arrayf32_to_points(pos.clone()),
          convert_flatu32_to_points(ind.clone()),
        ),
        // position: ColliderPosition::from([0.0, 100.0, 0.0]),
        ..ColliderBundle::default()
      };

      commands
        .spawn()
        .insert_bundle(rigid_body)
        .insert_bundle(collider)
        .insert_bundle(pbr)
        .insert(RigidBodyPositionSync::Discrete)
        // .insert(ColliderDebugRender::default())
        // .insert(ColliderPositionSync::Discrete)
        .insert(Player)
        ;
    }
  }
}

fn ground(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let size = 50.0;
  // let mesh = Mesh::from(mesh::shape::Plane { size: size });
  let mesh = Mesh::from(mesh::shape::Box::new(size, 0.1, size));
  let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);

  if let Some(VertexAttributeValues::Float3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
    if let Some(Indices::U32(ind)) = mesh.indices() {

      commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh.clone()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
      });
    
      let ground_size = 5.0;
      let ground_height = 0.1;
    
      let collider = ColliderBundle {
        // shape: ColliderShape::cuboid(ground_size, ground_height, ground_size),
        shape: ColliderShape::trimesh(
          convert_arrayf32_to_points(pos.clone()),
          convert_flatu32_to_points(ind.clone()),
        ),
        position: [0.0, 0.0, 0.0].into(),
        material: ColliderMaterial {
          friction: 1.0,
          ..Default::default()
        },
        ..ColliderBundle::default()
      };
      commands
        .spawn_bundle(collider)
        // .insert(ColliderDebugRender::default())
        .insert(ColliderPositionSync::Discrete);
      // println!("Create ground");
    }
  }

  // commands.spawn_bundle(PbrBundle {
  //   mesh: meshes.add(mesh.clone()),
  //   material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
  //   ..Default::default()
  // });

  // let ground_size = 20.0;
  // let ground_height = 0.1;

  // let collider = ColliderBundle {
  //   shape: ColliderShape::cuboid(ground_size, ground_height, ground_size),
  //   // shape: ColliderShape::trimesh(
  //   //   convert_arrayf32_to_points(pos.clone()),
  //   //   convert_flatu32_to_points(ind.clone()),
  //   // ),
  //   position: [-ground_size * 0.5, 0.0, -ground_size * 0.5].into(),
  //   ..ColliderBundle::default()
  // };
  // commands
  //   .spawn_bundle(collider)
  //   .insert(ColliderDebugRender::default())
  //   .insert(ColliderPositionSync::Discrete);
}

fn update(mut player_query: Query<(&Player, &RigidBodyPosition, &mut Transform)>) {
  for (_player, rigid_pos, mut trans) in player_query.iter_mut() {
    let pos = rigid_pos.position.translation.vector.xyz();
    // trans.translation = Vec3::new(pos[0], pos[1], pos[2]);
    // println!("r {:?} t {:?}", pos[0], trans.translation);
  }
}


pub fn convert_flatu32_to_points(array: Vec<u32>) -> Vec<[u32; 3]> {
  // let mut points: Vec<Point<u32>> = Vec::new();
  let mut points: Vec<[u32; 3]> = Vec::new();

  let slices: Vec<&[u32]> = array.chunks(3).collect();
  for slice in slices.iter() {
    // points.push(Point::new(slice[0], slice[1], slice[2]));
    // println!("ind {:?}", slice);
    points.push([slice[0], slice[1], slice[2]]);
  }
  points
}

pub fn convert_arrayf32_to_points(array: Vec<[f32; 3]>) -> Vec<Point<f32>> {
  let mut points: Vec<Point<f32>> = Vec::new();

  for array_p in array.iter() {
    // FIXME: Testing to move down the meshes to match the voxel
    // println!("pos {:?}", array_p);
    points.push(Point::new(array_p[0], array_p[1], array_p[2]));
  }
  points
}
