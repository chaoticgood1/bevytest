use bevy:: {
	prelude::*,
	render::{
		mesh::VertexAttribute, pipeline::PrimitiveTopology
	}
};

use bevy_rapier3d::rapier::{
	dynamics::{
		RigidBodyBuilder
	},
	geometry::{
		ColliderBuilder
	},
	math::{
		Point
	}
};

use bevy_rapier3d::physics::RapierPhysicsPlugin;
use bevy_rapier3d::render::RapierRenderPlugin;

#[derive(Clone, Debug)]
pub struct TriangleData {
	positions: Vec<[f32; 3]>,
	normals: Vec<[f32; 3]>,
	uvs: Vec<[f32; 2]>,
	indices: Vec<u32>,
}

fn main() {
	App::build()
		.add_resource(Msaa { samples: 4 })
		.add_default_plugins()
		.add_startup_system(setup.system())
		.add_plugin(RapierPhysicsPlugin)
		.add_plugin(RapierRenderPlugin)
		.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	create_bg(&mut commands, &mut meshes, &mut materials);
	create_collider_equilateral_triangle(&mut commands, &mut meshes, &mut materials, Vec3::new(0., 0., 0.))
}

fn create_bg(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	let ground_height = 0.1;
	let ground_size = 5.0;
	let ground_body = RigidBodyBuilder::new_static().translation(0.0, -ground_height, 0.0);
	let ground_collider = ColliderBuilder::cuboid(ground_size, 0.1, ground_size);

	commands
		.spawn((ground_body, ground_collider))
		.spawn(LightComponents {
			transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
			..Default::default()
		})
		.spawn(Camera3dComponents {
			transform: Transform::new(Mat4::face_toward(
				Vec3::new(0.0, 5.0, 18.0),
				Vec3::new(0.0, 0.0, 0.0),
				Vec3::new(0.0, 1.0, 0.0),
			)),
			..Default::default()
		});
}

fn create_collider_equilateral_triangle(
  commands: &mut Commands,
  _meshes: &mut ResMut<Assets<Mesh>>,
  _materials: &mut ResMut<Assets<StandardMaterial>>,
  pos: Vec3,
) {
	let data: TriangleData = triangle_data();
	let mesh = create_equilateral_triangle_mesh(data.clone());

  
	let body = RigidBodyBuilder::new_static().translation(pos.x(), pos.y(), pos.z());
	// let scale = Vec3::new(0.5, 0.5, 0.5);
	// let collider = ColliderBuilder::cuboid(scale.x(), scale.y(), scale.z());
	let collider = ColliderBuilder::trimesh(
		convert_arrayf32_to_points(data.positions),
		convert_flatu32_to_points(data.indices),
	);

	// commands
	// 	.spawn(PbrComponents {
  //     mesh: meshes.add(mesh),
  //     material: materials.add(Color::rgba(0.5, 0.4, 0.3, 1.0).into()),
  //     transform: Transform::from_non_uniform_scale(Vec3::new(1.0, 1.0, 1.0)),
  //     ..Default::default()
  //   });

	commands
		.spawn((body, collider));
}

fn create_equilateral_triangle_mesh(
	data: TriangleData
) -> Mesh {
  
  Mesh {
    primitive_topology: PrimitiveTopology::TriangleList,
    attributes: vec![
      VertexAttribute::position(data.positions),
      VertexAttribute::normal(data.normals),
      VertexAttribute::uv(data.uvs),
    ],
    indices: Some(data.indices),
  }
}

fn triangle_data() -> TriangleData {
	let size = 1.0;
  // Notes:
  // Plot the 3 dots
  // Dots degrees 0 120 240
  // Face degrees 60 180 300

  let top_point = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 0.0, 0.));
  let left_point = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 0.6667, 0.));
  let right_point = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 1.3333, 0.));

  let left_look_at = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 0.3333, 0.));
  let back_look_at = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 1.0, 0.));
  let right_look_at = rot_to_look_at(Vec3::new(0., std::f32::consts::PI * 1.6667, 0.));

  let top = [0., size, 0.];
  let bottom = [0., -size, 0.];
  let left = [left_look_at.x(), left_look_at.y(), left_look_at.z()];
  let right = [right_look_at.x(), right_look_at.y(), right_look_at.z()];
  let back = [back_look_at.x(), back_look_at.y(), back_look_at.z()];

  let half_height = size * 0.5;
  let vertices = &[
    // Top (0., size, 0.)
    (
      [top_point.x(), half_height, top_point.z()],
      top.clone(),
      [0., 0.],
    ),
    (
      [left_point.x(), half_height, left_point.z()],
      top.clone(),
      [-size, size],
    ),
    (
      [right_point.x(), half_height, right_point.z()],
      top.clone(),
      [size, size],
    ),
    // Bottom (0., -size, 0.)
    (
      [top_point.x(), -half_height, top_point.z()],
      bottom.clone(),
      [0., 0.],
    ),
    (
      [right_point.x(), -half_height, right_point.z()],
      bottom.clone(),
      [size, size],
    ),
    (
      [left_point.x(), -half_height, left_point.z()],
      bottom.clone(),
      [size, -size],
    ),
    // Left
    (
      [top_point.x(), half_height, top_point.z()],
      left.clone(),
      [0., 0.],
    ),
    (
      [top_point.x(), -half_height, top_point.z()],
      left.clone(),
      [0., 0.],
    ),
    (
      [left_point.x(), -half_height, left_point.z()],
      left.clone(),
      [0., 0.],
    ),
    (
      [left_point.x(), half_height, left_point.z()],
      left.clone(),
      [0., 0.],
    ),
    // Right
    (
      [top_point.x(), half_height, top_point.z()],
      right.clone(),
      [0., 0.],
    ),
    (
      [right_point.x(), half_height, right_point.z()],
      right.clone(),
      [0., 0.],
    ),
    (
      [right_point.x(), -half_height, right_point.z()],
      right.clone(),
      [0., 0.],
    ),
    (
      [top_point.x(), -half_height, top_point.z()],
      right.clone(),
      [0., 0.],
    ),
    // Back
    (
      [left_point.x(), half_height, left_point.z()],
      back.clone(),
      [0., 0.],
    ),
    (
      [left_point.x(), -half_height, left_point.z()],
      back.clone(),
      [0., 0.],
    ),
    (
      [right_point.x(), -half_height, right_point.z()],
      back.clone(),
      [0., 0.],
    ),
    (
      [right_point.x(), half_height, right_point.z()],
      back.clone(),
      [0., 0.],
    ),
  ];

  let mut positions = Vec::new();
  let mut normals = Vec::new();
  let mut uvs = Vec::new();
  for (position, normal, uv) in vertices.iter() {
    positions.push(*position);
    normals.push(*normal);
    uvs.push(*uv);
  }

  let indices = vec![
    0, 1, 2, // top
    3, 4, 5, // Bottom
    6, 7, 8, 6, 8, 9, // Left
    10, 11, 12, 10, 12, 13, // Right
    14, 15, 16, 14, 16, 17, // Back
	];
	
	TriangleData {
		positions: positions,
		normals: normals,
		uvs: uvs,
		indices: indices
	}
}

fn convert_arrayf32_to_points(array: Vec<[f32; 3]>) -> Vec<Point<f32>> {
	let mut points: Vec<Point<f32>> = Vec::new();
	
	for array_p in array.iter() {
		points.push(Point::new(array_p[0], array_p[1], array_p[2]));
	}
	points
}

fn convert_flatu32_to_points(array: Vec<u32>) -> Vec<Point<u32>> {
	let mut points: Vec<Point<u32>> = Vec::new();

	let slices: Vec<&[u32]> = array.chunks(3).collect();
	for slice in slices.iter() {
		points.push(Point::new(slice[0], slice[1], slice[2]));
	}
	points
}

pub fn rot_to_look_at(rot: Vec3) -> Vec3 {
	let yaw = rot.y() - std::f32::consts::PI * 0.5;

	let len = rot.x().cos();
	return Vec3::new(yaw.cos() * len, rot.x().sin(), -yaw.sin() * len).normalize();
}