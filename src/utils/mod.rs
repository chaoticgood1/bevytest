use bevy::math::{Vec3, Quat};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use voxels::{chunk::{voxel_pos_to_key}, data::{voxel_octree::VoxelOctree, surface_nets::{VoxelReuse, get_surface_nets2}}};

pub struct Math;

impl Math {
  pub fn look_at_to_rotation_quat(look_at: Vec3) -> Quat {
    let rot = Math::look_at_to_rotation(look_at);
    // Quat::from_rotation_ypr(rot.y, rot.x, 0.0)
    Quat::from_rotation_y(rot.y) * Quat::from_rotation_x(rot.x)
  }

  pub fn look_at_to_rotation(look_at: Vec3) -> Vec3 {
    let tmp_look_at = look_at.normalize();
    let mut rad_x = tmp_look_at.y;
    if rad_x.is_nan() {
      rad_x = 0.0;
    }

    let mut rad_y = tmp_look_at.x / tmp_look_at.z;
    if rad_y.is_nan() {
      rad_y = 0.0;
    }

    let mut y_rot = rad_y.atan();
    if tmp_look_at.z > 0.0 {
      let half_pi = std::f32::consts::PI * 0.5;
      y_rot = -((half_pi) + (half_pi - y_rot));
    }

    Vec3::new(rad_x.asin(), y_rot, 0.0)
  }

  pub fn rot_to_look_at(rot: Vec3) -> Vec3 {
    let yaw = rot.y - std::f32::consts::PI * 0.5;

    let len = rot.x.cos();
    return Vec3::new(yaw.cos() * len, rot.x.sin(), -yaw.sin() * len).normalize();
  }
}

pub fn to_key(translation: &Vec3, seamless_size: u32) -> [i64; 3] {
  let pos = [translation.x as i64, translation.y as i64, translation.z as i64];
  voxel_pos_to_key(&pos, seamless_size)
}

pub fn create_collider_mesh(
  octree: &VoxelOctree, 
  voxel_reuse: &mut VoxelReuse
) -> MeshColliderData {
  let mesh = get_surface_nets2(octree, voxel_reuse);

  let mut positions = Vec::new();
  let mut indices = Vec::new();
  
  for pos in mesh.positions.iter() {
    // positions.push(Point::new(pos[0], pos[1], pos[2]));
    positions.push(Vec3::new(pos[0], pos[1], pos[2]));
  }
  
  for ind in mesh.indices.chunks(3) {
    // println!("i {:?}", ind);
    indices.push([ind[0], ind[1], ind[2]]);
  }


  MeshColliderData {
    positions: positions,
    indices: indices,
  }
}

#[derive(Clone)]
pub struct MeshColliderData {
  // pub positions: Vec<Point<f32>>,
  pub positions: Vec<Vec3>,
  pub indices: Vec<[u32; 3]>,
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



pub fn key_to_world_coord_f32(key: &[i64; 3], seamless_size: u32) -> [f32; 3] {
  [
    (key[0] * seamless_size as i64) as f32,
    (key[1] * seamless_size as i64) as f32,
    (key[2] * seamless_size as i64) as f32,
  ]
}
