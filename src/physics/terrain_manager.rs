/*
  TODO
    Defer:
      Have to simplify the naming convention later
      Implement caching of chunk data and mesh
    Current:
      Implementation of terrain editing
        Remove preloading terrain
          As everything can be edited, therefore everything will be edited
        Identify the chunk coord
          Raycast collider
          Identify world coord
          Convert to chunk coord
        Edit the chunk data in chunk coord
        Save
          Data
            Mesh
              Collider(Just have to convert to the right data structure)
              Graphics
          

        Implement management of chunk data, collider mesh and mesh for rendering

        Raycast to the collider
        Then identify the data being hit(Needed to know exactly the value later)
        Identification by world pos
        Then convert to chunk pos
      
*/



use std::thread;

use bevy::{prelude::*, utils::HashMap};
use flume::{Sender, Receiver};
use voxels::chunk::chunk_manager::{Chunk, Configuration, ChunkManager};

use super::{Physics, player::Player};

#[derive(Clone)]
pub struct TerrainManager {
  pub data: HashMap<[i64; 3], Chunk>,
  send_load_keys: Sender<Vec<[i64; 3]>>,
  send_keys: Sender<Vec<([i64; 3], Chunk)>>,

  pub receive_load_keys: Receiver<Vec<([i64; 3], Chunk)>>,
  receive_keys: Receiver<Vec<[i64; 3]>>,
  
  pub call_count: u32,
  pub fulfilled_call_count: u32,
}

impl TerrainManager {
  /*
    TODO
      Spawn async
        Load data
        Load mesh
      Wait for it to be finished
      
  */

  pub fn new() -> Self {
    let channel = 100;
    let (send_load_keys, receive_keys) = flume::bounded::<Vec<[i64; 3]>>(channel);
    let (send_keys, receive_load_keys) = flume::bounded::< Vec<([i64; 3], Chunk )> >(channel);
    Self { 
      data: HashMap::new(),
      send_load_keys: send_load_keys,
      send_keys: send_keys,
      receive_load_keys: receive_load_keys,
      receive_keys: receive_keys,
      call_count: 0,
      fulfilled_call_count: 0,
    }
  }

  pub fn reset(&mut self) {
    self.call_count = 0;
    self.fulfilled_call_count = 0;
  }

  pub fn done_loading(&mut self) -> bool {
    self.call_count >= self.fulfilled_call_count
  }

  pub fn load_data(
    &mut self,
    config: &Configuration, 
    keys: &Vec<[i64; 3]>,
  ) {
    self.call_count += 1;
    let sender = self.send_keys.clone();
    let keys = keys.clone();
    let depth = config.depth;
    let noise = config.noise.clone();
    thread::spawn(move || {
      let mut result = Vec::new();
      for key in keys.iter() {
        result.push(
          (key.clone(), ChunkManager::new_chunk(key, depth, depth, noise))
        );
      }
      sender.send(result);
      // println!("Testing1");
    });

    
    // if keys.len() > 0 {
    //   player.newly_added_keys.clear();
    // }
    // for key in keys.iter() {
    //   let data = collider_data.get(key).unwrap();
  
    //   if data.indices.len() == 0 { // Temporary, should be removed once the ChunkMode detection is working
    //     continue;
    //   }
  
    //   let pos_f32 = key_to_world_coord_f32(key, config.seamless_size);
    //   let mut pos = Vec::new();
    //   for d in data.positions.iter() {
    //     pos.push(Point::from([d.x, d.y, d.z]));
    //   }
  
    //   let mut collider = ColliderBuilder::trimesh(pos, data.indices.clone())
    //     .collision_groups(InteractionGroups::new(1, 2))
    //     .build();
    //   collider.set_position(Isometry::from(pos_f32));
      
    //   player.terrain_colliders.push(
    //     TerrainCollider {
    //       key: key.clone(),
    //       collider_handle: physics.collider_set.insert(collider)
    //     }
    //   );
    //   player.newly_added_keys.push(key.clone());
    //   // println!("spawn_colliders {:?}", key);
    // }
  }
}


#[cfg(test)]
mod tests {
  use voxels::chunk::chunk_manager::ChunkManager;
  use super::TerrainManager;


  #[test]
  fn test_manager() -> Result<(), String> {
    let mut manager = TerrainManager::new();
    let config = ChunkManager::default().config.clone();

    let keys = vec![
      [0, 0, 0],
      [1, 0, 0],
    ];
    manager.load_data(&config, &keys.to_vec());

    loop {
      let len = manager.receive_load_keys.len();
      for chunks in manager.receive_load_keys.drain() {
        for data in chunks.iter() {
          println!("data {:?}", data.0);
        }
      }
      if len > 0 {
        break;
      }
      

    };
    Ok(())
  }


}
