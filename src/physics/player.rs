use bevy::prelude::*;
use rapier3d::prelude::{Collider, ColliderHandle, RigidBodyHandle};
use crate::utils::to_key;
use super::manager::KeyEvents;

#[derive(Clone)]
pub struct Player {
  pub config: PlayerConfig,
  pub key_events: KeyEvents,
  pub key_pressed: KeyEvents,
  pub forward: Vec3,
  pub collider: Collider,
  pub collider_handle: ColliderHandle,
  pub terrain_colliders: Vec<TerrainCollider>,
  pub newly_added_keys: Vec<[i64; 3]>,
  pub pos: Vec3,
}

impl Player {
  pub fn new(
    config: PlayerConfig, 
    collider: Collider, 
    collider_handle: ColliderHandle
  ) -> Self {
    Self {
      config: config,
      key_events: KeyEvents::default(),
      key_pressed: KeyEvents::default(),
      forward: Vec3::ZERO,
      collider: collider,
      collider_handle: collider_handle,
      terrain_colliders: Vec::new(),
      newly_added_keys: Vec::new(),
      pos: Vec3::ZERO
    }
  }
}


#[derive(Default, Component, Clone)]
pub struct PlayerConfig {
  pub speed: f32,
  pub prev_key: [i64; 3],
  pub cur_key: [i64; 3],
  pub peer_id: String,
  pub body_handle: RigidBodyHandle,
  pub pitch: f32,
  pub yaw: f32,

  has_moved: bool,
  newly_added: bool,
  is_updated: bool,
}

impl PlayerConfig {
  pub fn new(
    peer_id: String,
    handle: RigidBodyHandle,
    pos: Vec3, 
    seamless_size: u32
  ) -> Self {
    Self {
      speed: 5.0,
      prev_key: [i64::MIN, i64::MIN, i64::MIN],
      cur_key: to_key(&pos, seamless_size),
      peer_id: peer_id.clone(),
      body_handle: handle,
      pitch: 0.0,
      yaw: 180.0,

      has_moved: false,
      newly_added: true,
      is_updated: false,
    }
  }

  pub fn newly_added(&self) -> bool {
    self.newly_added
  }

  pub fn has_moved(&self) -> bool {
    self.has_moved && !self.newly_added
  }

  pub fn update(&mut self, seamless_size: u32, pos: &Vec3) {
    let key = to_key(pos, seamless_size);

    if self.cur_key == key {
      self.has_moved = false;
      // println!("self.newly_added {}", self.newly_added);
    }
    
    if self.cur_key != key {
      self.prev_key = self.cur_key.clone();
      self.cur_key = key;
      self.has_moved = true;
    }

    self.update_newly_added_status();
  }

  fn update_newly_added_status(&mut self) {
    if self.is_updated {
      self.newly_added = false;
    }
    if !self.is_updated {
      self.is_updated = true;
    }
  }
}

#[derive(Clone)]
pub struct TerrainCollider {
  pub key: [i64; 3],
  pub collider_handle: ColliderHandle,
}
