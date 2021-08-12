pub mod camera;
pub mod player;
use crate::camera::fly_camera::FlyCamera;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
  App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(camera::CustomPlugin)
    .add_plugin(player::CustomPlugin)
    .run();
}

pub fn create_mesh(
  meshes: &mut ResMut<Assets<Mesh>>,
  positions: Vec<[f32; 3]>,
  normals: Vec<[f32; 3]>,
  indices: Vec<u32>,
) -> Handle<Mesh> {
  use bevy::render::mesh::Indices;
  use bevy::render::pipeline::PrimitiveTopology;

  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  let mut uvs = Vec::new();
  for _i in 0..normals.len() {
    uvs.push([0, 0]);
  }
  mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
  mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
  mesh.set_indices(Some(Indices::U32(indices)));
  meshes.add(mesh)
}