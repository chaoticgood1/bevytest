use bevy::prelude::*;
use rapier3d::{prelude::{RigidBodySet, ColliderSet, PhysicsPipeline, ColliderBuilder, RigidBodyBuilder, Real, Vector, IntegrationParameters, IslandManager, PhysicsHooks, EventHandler, MultibodyJointSet, ImpulseJointSet, NarrowPhase, BroadPhase, CCDSolver, RigidBodyHandle, Collider, ColliderHandle, InteractionGroups}, na::Vector3};

pub mod manager;
pub mod player;
pub mod system;
pub mod terrain_manager;

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(manager::CustomPlugin)
      .add_plugin(system::CustomPlugin)
      .insert_resource(Physics::default());
  }
}

pub struct Physics {
  pub pipeline: PhysicsPipeline,
  pub rigid_body_set: RigidBodySet,
  pub collider_set: ColliderSet,
  pub gravity: Vector<Real>,
  pub integration_parameters: IntegrationParameters,
  pub island_manager: IslandManager,
  pub broad_phase: BroadPhase,
  pub narrow_phase: NarrowPhase,
  pub impulse_joint_set: ImpulseJointSet,
  pub multibody_joint_set: MultibodyJointSet,
  pub ccd_solver: CCDSolver,
  pub physics_hooks: Box<dyn PhysicsHooks>,
  pub event_handler: Box<dyn EventHandler>,
}

impl Default for Physics {
  fn default() -> Self {
    Self {
      pipeline: PhysicsPipeline::new(),
      rigid_body_set: RigidBodySet::new(),
      collider_set: ColliderSet::new(),
      gravity: Vector::y() * -9.81,
      integration_parameters: IntegrationParameters {
        dt: 1.0 / 30.0,
        ..default()
      },
      island_manager: IslandManager::new(),
      broad_phase: BroadPhase::new(),
      narrow_phase: NarrowPhase::new(),
      impulse_joint_set: ImpulseJointSet::new(),
      multibody_joint_set: MultibodyJointSet::new(),
      ccd_solver: CCDSolver::new(),
      physics_hooks: Box::new(()),
      event_handler: Box::new(()),
    }
  }
}

impl Physics {
  pub fn step(&mut self) {
    self.pipeline.step(
        &self.gravity, 
        &self.integration_parameters, 
        &mut self.island_manager, 
        &mut self.broad_phase, 
        &mut self.narrow_phase, 
        &mut self.rigid_body_set, 
        &mut self.collider_set, 
        &mut self.impulse_joint_set, 
        &mut self.multibody_joint_set, 
        &mut self.ccd_solver, 
        self.physics_hooks.as_mut(), 
        self.event_handler.as_mut()
      );
  }

  pub fn spawn_character(&mut self, depth: f32, radius: f32, pos: Vec3) -> (RigidBodyHandle, ColliderHandle) {
    let collider = ColliderBuilder::capsule_y(depth * 0.5, radius)
      .collision_groups(InteractionGroups::new(2, 1))
      .build();
    let rigid_body = RigidBodyBuilder::dynamic()
      .translation(Vector3::from([pos.x, pos.y, pos.z]))
      .lock_rotations()
      .build();
    let body_handle = self.rigid_body_set.insert(rigid_body);
    let collider_handle = self.insert_with_parent(collider, body_handle.clone());
    (body_handle, collider_handle)
  }

  pub fn insert_with_parent(&mut self, collider: Collider, handle: RigidBodyHandle) -> ColliderHandle {
    self
      .collider_set
      .insert_with_parent(collider, handle, &mut self.rigid_body_set)
  }

  pub fn remove_collider(&mut self, handle: ColliderHandle) {
    self
      .collider_set
      .remove(handle, &mut self.island_manager, &mut self.rigid_body_set, true);
  }
}