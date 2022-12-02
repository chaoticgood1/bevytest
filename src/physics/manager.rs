use std::{fs, collections::VecDeque};
use bevy::prelude::*;
use bevy::utils::HashMap;
use rapier3d::{prelude::{ColliderHandle, ColliderBuilder, Vector, RigidBodySet, ColliderSet, IntegrationParameters, IslandManager, BroadPhase, NarrowPhase, ImpulseJointSet, MultibodyJointSet, CCDSolver, InteractionGroups}, na::{Point, Isometry}};
// use tokio::sync::mpsc::{Sender, Receiver};
use ironverse::tokio::sync::mpsc::{Sender, Receiver};
use voxels::{chunk::{chunk_manager::{Configuration, ChunkManager, Chunk}, adjacent_keys, adj_delta_keys, is_adjacent}, data::{surface_nets::VoxelReuse, voxel_octree::VoxelMode}};
use serde::{Deserialize, Serialize};
use crate::{utils::{create_collider_mesh, MeshColliderData, key_to_world_coord_f32, to_key}};
use super::{Physics, player::{Player, PlayerConfig, TerrainCollider}, system::{Output, Input}, terrain_manager::TerrainManager};

pub const DEPTH: f32 = 1.0;
pub const RADIUS: f32 = 1.0;

#[derive(Clone)]
pub struct Data {
  pub event: Event,
  pub players: Vec<Player>,
}

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<Spawn>()
      .add_event::<Event>()
      .insert_resource(PhysicsManager::default());
  }
}

#[derive(Clone, Component)]
pub struct PhysicsManager {
  pub tick: u128,
  pub collider_data: HashMap<[i64; 3], MeshColliderData>,
  pub collider_handles: HashMap<[i64; 3], ColliderHandle>,
  pub players: Vec<Player>,
  pub events: VecDeque<Event>,
  pub saved_events: Vec<Event>,
  pub current_event: Option<Event>,
  pub chunks: HashMap<[i64; 3], Chunk>,
  pub data: Vec<Data>,

  pub terrain_manager: TerrainManager,
}

impl Default for PhysicsManager {
  fn default() -> Self {
    Self {
      tick: 0,
      collider_data: HashMap::default(),
      collider_handles: HashMap::default(),
      players: Vec::new(),
      events: VecDeque::new(),
      saved_events: Vec::new(),
      current_event: None,
      chunks: HashMap::default(),
      data: Vec::new(),
      terrain_manager: TerrainManager::new(),
    }
  }
}

impl PhysicsManager {
  // Deprecated: In favor of loading on the fly to make the simulation work
  pub fn load_terrains(
    &mut self,
    config: &Configuration,
  ) {
    let range = 5;

    let mut voxel_reuse = config.voxel_reuse.clone();
    for x in -range..range {
      for y in -range..range {
        for z in -range..range {
          let key = [x, y, z];
          // println!("key {:?}", key);

          let chunk = ChunkManager::new_chunk(&key, config.depth, config.lod, config.noise);
          self.chunks.insert(key.clone(), chunk.clone());

          let data = create_collider_mesh(&chunk.octree, &mut voxel_reuse);
          self.collider_data.insert(key.clone(), data);
        }
      }
    }
  }
// 
  pub fn add_player(
    &mut self, 
    physics: &mut Physics,
    config: &Configuration,
    event: &Spawn,
  ) -> PlayerConfig {
    let (handle, collider_handle) = physics.spawn_character(DEPTH, RADIUS, Vec3::from(event.pos));
    let player = Player::new(
      PlayerConfig::new(
        event.peer_id.clone(), 
        handle, 
        Vec3::new(event.pos[0], event.pos[1], event.pos[2]),
        config.seamless_size
      ),
      physics.collider_set.get(collider_handle).unwrap().clone(),
      collider_handle.clone(),
    );

   
    self.players.push(player.clone());
    player.config.clone()
  }

  pub async fn sim_update(
    &mut self, 
    physics: &mut Physics,
    config: &Configuration,
    input_rcv: &mut Receiver<Input>,
    output: &Sender<Output>,
    frames: &mut u128
  ) {
    loop {
      match input_rcv.try_recv() {
        Ok(input) => {
          if input.events.len() > 0 {
            self.events.append(&mut input.events.clone());
          }
          
        }
        Err(_e) => { break; } // FIXME: Handle error later
      }
    }

    let current_op = self.current_event.clone();
    if self.events.len() > 0 {

      if current_op.is_none() {
        if self.tick <= self.events[0].tick {
          self.current_event = self.events.pop_front();
        } else {
          self.events.pop_front();
        } 

      } else {
        let event = current_op.clone().unwrap();
        if self.tick > event.tick {
          self.current_event = self.events.pop_front();
        }
      }
    }

    let current_op = self.current_event.clone();
    if current_op.is_some() {
      let event = current_op.unwrap();
      // println!("event.tick {}", event.tick);

      if self.tick <= event.tick {
        *frames += 1;


        // Have to wait if this is done calculating terrains
        self.update(physics, config).await;
        physics.step();

        let mut e = event.clone();
        if e.tick != self.tick - 1 {
          e.spawns.clear();
          e.actions.clear();
        }
        let _ = output.send(Output { 
          event: e,
          players: self.players.clone()
        }).await;
      } else {
        self.current_event = None;
      }

    }
  }


  // TODO: Rename to physics update later
  async fn update(&mut self, physics: &mut Physics, config: &Configuration) {
    if self.current_event.is_none() {
      return;
    }
    let event = self.current_event.clone().unwrap();
    if self.tick == event.tick {
      // println!("self.tick == event.tick {} {}", self.tick, event.actions.len());
      self.process_spawns(physics, config, &event.spawns);
      self.process_actions(&event.actions);
    }
    self.process_player_keys(physics, config);
    self.process_rigidbodies(physics);
    self.on_modified_terrains(physics, config);
    self.process_movement_terrains(physics, config).await;
    self.tick += 1;
  }

  fn process_spawns(&mut self, physics: &mut Physics, config: &Configuration, spawns: &Vec<Spawn>) {
    for spawn in spawns.iter() {
      let _ = self.add_player(physics, config, spawn);
    }
  }

  fn process_actions(&mut self, all_actions: &Vec<Actions>) {
    for player in self.players.iter_mut() {
      player.key_pressed.events.clear();
    }

    'main: for actions in all_actions.iter() {
      for player in self.players.iter_mut() {
        
        if actions.peer_id == player.config.peer_id {
          let len = player.key_events.events.len();
          for index in (0..len).rev() {
            let event = player.key_events.events[index].clone();

            for new_event in actions.events.iter() {
              if event.key == new_event.key && new_event.key_state == KeyState::Up {
                player.key_events.events.swap_remove(index);
              }
            }
          }

          for event in actions.events.iter() {
            if event.key_state == KeyState::Down {
              player.key_events.events.push(event.clone());
            }
            if event.key_state == KeyState::Pressed {
              player.key_pressed.events.push(event.clone());
            }
          }
          player.forward = actions.forward.clone();
          continue 'main;
        }
      }
    }
  }

  /*
    Defer optimization, make it work

    Have similarities when editing the movement terrains
    Simplify the requirements
    Overview what's necessary to accomplish
    Won't be able to edit it without existing terrain data
    Therefore it is inserting new data, not loading the default data

  */
  fn on_modified_terrains(&mut self, physics: &mut Physics, config: &Configuration) {
    /*
      Get player view direction
      Identify which keys it is hitting
        I need raycast
      Identify if there is a chunk there
      Save the data to terrain manager
      Remove existing chunks by keys
      Add modified chunk
    */
    for player in self.players.iter_mut() {
      // Need to know the coordinate to modify
      'keys_loop: for ev in player.key_pressed.events.iter() {
        if ev.key == KeyCode::E as u32 {
          // let rigid_body = physics.rigid_body_set.get_mut(player.config.body_handle).unwrap();
          // let trans = rigid_body.translation();
          // let pos = [trans.x, trans.y, trans.z];
          // let keys = get_raycast(&pos, &player.forward, 4, config.seamless_size);

          // println!("Add terrain {:?}", keys);
          // for key in keys.iter() {
          //   let chunk_op = self.terrain_manager.data.get(key);
          //   if chunk_op.is_some() {
          //     println!("Exists {:?}", key);

          //     // How to get the specific local chunk coordinate then?
              
          //     continue 'keys_loop;
          //   }
          // }
        }
      }
    }


  }



  async fn process_movement_terrains(&mut self, physics: &mut Physics, config: &Configuration) {
    self.terrain_manager.reset();

    let mut keys = Vec::new();
    let collider_data = &self.collider_data;
    for player in self.players.iter_mut() {
      if player.config.newly_added() {
        keys = adjacent_keys(&player.config.cur_key, 1);
      }

      if player.config.has_moved() {
        keys = adj_delta_keys(&player.config.prev_key, &player.config.cur_key, 1);
      }
      self.terrain_manager.load_data(config, &keys);
      // spawn_colliders(collider_data, physics, config, &keys, player);
    }

    loop {
      let receiver = self.terrain_manager.receive_load_keys.clone();
      for data in receiver.drain() {
        for (key, chunk) in data.iter() {
          self.terrain_manager.data.insert(key.clone(), chunk.clone());
        }

        spawn_colliders(data, physics, config, &keys);
        // println!("spawn_colliders");

        if self.terrain_manager.done_loading() {
          // println!("done_loading");
          return;
        }
      }
    }
    // println!("self.tick1 {}", self.tick);


    // let mut keys = Vec::new();
    // let collider_data = &self.collider_data;
    // for player in self.players.iter_mut() {
    //   if player.config.newly_added() {
    //     keys = adjacent_keys(&player.config.cur_key, 1);
    //   }

    //   if player.config.has_moved() {
    //     keys = adj_delta_keys(&player.config.prev_key, &player.config.cur_key, 1);
    //   }
    //   spawn_colliders(collider_data, physics, config, &keys, player);
    // }
    // despawn_colliders(self, physics, config);
  }



  fn process_player_keys(&mut self, physics: &mut Physics, config: &Configuration) {
    for player in self.players.iter_mut() {
      let rigid_body = &physics.rigid_body_set[player.config.body_handle];
      let trans = rigid_body.translation();
      let pos = Vec3::new(trans.x, trans.y, trans.z);
      player.pos = pos.clone();
      player.config.update(config.seamless_size, &pos);
    }
  }

  fn process_rigidbodies(&mut self, physics: &mut Physics) {
    for player in self.players.iter_mut() {
      let events = &player.key_events.events;
      if events.len() > 0 {
        // println!("key_events {:?} look_at {:?}: {:?}", e.events, look_at.forward, look_at.right);
      
        let forward = player.forward;
        let mut f = forward.clone();
        f.y = 0.0;
        f = f.normalize();
        let right = f.cross(Vec3::Y);
        let mut direction = Vec3::ZERO;

        for ev in events.iter() {
          if ev.key == KeyCode::W as u32 {
            direction += f;
          }
          if ev.key == KeyCode::S as u32 {
            direction += f * -1.0;
          }
          if ev.key == KeyCode::A as u32 {
            direction += right * -1.0;
          }
          if ev.key == KeyCode::D as u32 {
            direction += right;
          }
        }

        if direction != Vec3::ZERO {
          let effort = 200.0; // Default effort needed for integration test
          // let effort = 150.0;
          let dt = physics.integration_parameters.dt;
          let interp_force = direction.normalize() * effort * dt;

          let rigid_body = physics.rigid_body_set.get_mut(player.config.body_handle).unwrap();
          rigid_body.apply_impulse(
            Vector::from([interp_force.x, interp_force.y, interp_force.z]), true
          );
          // println!("sim move {:?}: {:?}", direction, rigid_body.translation());
        }
        
      }
      // let rigid_body = physics.rigid_body_set.get_mut(player.config.body_handle).unwrap();
      // println!("sim move {:?}: {:?}", tick, rigid_body.translation());
      // println!("{:?}", rigid_body.translation());
    }
  }
  

  pub fn is_done(&self) -> bool {
    // self.events.is_empty() && self.tick > self.current_event.tick
    true
  }

  pub fn process_result(&mut self, physics: &mut Physics) -> Result {
    Result {
      physics_manager: self.clone(),
      rigid_body_set: physics.rigid_body_set.clone(),
      collider_set: physics.collider_set.clone(),
      integration_parameters: physics.integration_parameters.clone(),
      island_manager: physics.island_manager.clone(),
      broad_phase: physics.broad_phase.clone(),
      narrow_phase: physics.narrow_phase.clone(),
      impulse_joint_set: physics.impulse_joint_set.clone(),
      multibody_joint_set: physics.multibody_joint_set.clone(),
      ccd_solver: physics.ccd_solver.clone(),
    }
  }


  pub fn copy(&mut self, physics: &mut Physics, result: &Result) {
    let manager = &result.physics_manager;
    self.tick = manager.tick.clone();
    self.collider_data = manager.collider_data.clone();
    self.collider_handles = manager.collider_handles.clone();
    self.players = manager.players.clone();
    self.events = manager.events.clone();
    self.saved_events = manager.saved_events.clone();
    self.current_event = manager.current_event.clone();
    self.chunks = manager.chunks.clone();

    physics.rigid_body_set = result.rigid_body_set.clone();
    physics.collider_set = result.collider_set.clone();
    physics.integration_parameters = result.integration_parameters.clone();
    physics.island_manager = result.island_manager.clone();
    physics.broad_phase = result.broad_phase.clone();
    physics.narrow_phase = result.narrow_phase.clone();
    physics.impulse_joint_set = result.impulse_joint_set.clone();
    physics.multibody_joint_set = result.multibody_joint_set.clone();
    physics.ccd_solver = result.ccd_solver.clone();
  }

}

fn spawn_colliders(
  chunks: Vec<([i64; 3], Chunk)>,
  physics: &mut Physics, 
  config: &Configuration, 
  keys: &Vec<[i64; 3]>,
) {
  let mut voxel_reuse = config.voxel_reuse.clone();
  for (key, chunk) in chunks.iter() {
    let data = chunk.octree.compute_mesh2(VoxelMode::SurfaceNets, &mut voxel_reuse);

    if data.indices.len() == 0 { // Temporary, should be removed once the ChunkMode detection is working
      continue;
    }

    let pos_f32 = key_to_world_coord_f32(key, config.seamless_size);
    let mut pos = Vec::new();
    for d in data.positions.iter() {
      pos.push(Point::from([d[0], d[1], d[2]]));
    }

    let mut indices = Vec::new();
    for ind in data.indices.chunks(3) {
      // println!("i {:?}", ind);
      indices.push([ind[0], ind[1], ind[2]]);
    }

    let mut collider = ColliderBuilder::trimesh(pos, indices)
      .collision_groups(InteractionGroups::new(1, 2))
      .build();
    collider.set_position(Isometry::from(pos_f32));

    physics.collider_set.insert(collider);
    
    // player.terrain_colliders.push(
    //   TerrainCollider {
    //     key: key.clone(),
    //     collider_handle: physics.collider_set.insert(collider)
    //   }
    // );
    // player.newly_added_keys.push(key.clone());
    // println!("spawn_colliders {:?}", key);
  }




  
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
  //   // player.newly_added_keys.push(key.clone());
  //   // println!("spawn_colliders {:?}", key);
  // }
}


fn despawn_colliders(
  manager: &mut PhysicsManager,
  physics: &mut Physics, 
  config: &Configuration,
) {

  for player in manager.players.iter_mut() {
    let rigid_body = physics.rigid_body_set.get_mut(player.config.body_handle).unwrap();
    let t = rigid_body.translation();
    let key = &to_key(&Vec3::new(t.x, t.y, t.z), config.seamless_size);

    // TODO: Continue despawn implementation
    let len = player.terrain_colliders.len();
    for index in (0..len).rev() {
      let terrain_col = player.terrain_colliders[index].clone();
      if !is_adjacent(key, &terrain_col.key) {
        player.terrain_colliders.swap_remove(index);
        physics.remove_collider(terrain_col.collider_handle);
      }
    }
  }
}

#[derive(Component, Clone)]
pub struct KeyEvents {
  pub events: Vec<KeyEvent>,
}

impl Default for KeyEvents {
  fn default() -> Self {
    Self {
      events: Vec::new(),
    }
  }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Event {
  pub tick: u128,
  pub spawns: Vec<Spawn>,
  pub actions: Vec<Actions>
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Spawn {
  // pub id: u32,  // If ID is 0, new player
  pub peer_id: String,
  pub pos: [f32; 3]
}

impl Default for Spawn {
  fn default() -> Self {
    Self {
      peer_id: "".to_string(),
      pos: [0.0, 0.0, 0.0],
    }
  }
}



#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Actions {
  pub peer_id: String,
  pub events: Vec<KeyEvent>,
  pub forward: Vec3,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct KeyEvent {
  pub key: u32,
  pub key_state: KeyState, 
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, Debug, Hash)]
pub enum KeyState {
  Up,
  Down,
  Pressed,
}


#[derive(Component)]
pub struct Result {
  pub physics_manager: PhysicsManager,
  pub rigid_body_set: RigidBodySet,
  pub collider_set: ColliderSet,
  pub integration_parameters: IntegrationParameters,
  pub island_manager: IslandManager,
  pub broad_phase: BroadPhase,
  pub narrow_phase: NarrowPhase,
  pub impulse_joint_set: ImpulseJointSet,
  pub multibody_joint_set: MultibodyJointSet,
  pub ccd_solver: CCDSolver,
}

// System Design
/* Only to indicate order execution as using Custom Stage have following cons:
 *    - Events not guaranteed to be called
 *    - Order execution not guaranteed when switching between State
 *  Reason why needed this anchor function order execution:
 *    To separate features into plugins for overall faster development vs monolithic code structure
 *  
 *  Have to review if this pending proposal progresses: https://github.com/alice-i-cecile/rfcs/blob/stageless/rfcs/45-stageless.md
 */