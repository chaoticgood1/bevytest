use std::{time::Duration, collections::VecDeque};
use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, Task}, ecs::schedule::ShouldRun};
use ironverse::tokio::sync::mpsc::{self, Sender, Receiver};
use voxels::chunk::chunk_manager::ChunkManager;
// use crate::client::{GameResource};
use futures_lite::future;
use super::{manager::{PhysicsManager, Event}, Physics, player::Player};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum SimulationStage {
  PreUpdate,
  Update,
  PostUpdate,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationState {
  None,
  Start,
}

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(PhysicsResource::default())
      .add_state(SimulationState::Start)
      .add_stage_before(CoreStage::Update, SimulationStage::PreUpdate, SystemStage::parallel())
      .add_stage_after(CoreStage::Update, SimulationStage::Update, SystemStage::parallel())
      .add_stage_before(CoreStage::PostUpdate, SimulationStage::PostUpdate, SystemStage::parallel())
      .add_system_set_to_stage(
        SimulationStage::PreUpdate,
        SystemSet::new()
          .with_system(process_output)
          .with_system(process_output_done)
          .with_run_criteria(is_simulation_enabled)
      )
      .add_system_set_to_stage(
        SimulationStage::PostUpdate,
        SystemSet::new()
          .with_system(process_input)
          .with_system(process_input_done)
          .with_run_criteria(is_simulation_enabled)
      )
      .add_system_set(
        SystemSet::on_enter(SimulationState::Start)
          .with_system(init)
          .with_system(start_simulator.after(init))
      )
      ;
  }
}

pub fn is_simulation_enabled(
  state: Res<State<SimulationState>>,
) -> ShouldRun {
  if state.current() == &SimulationState::Start {
    return ShouldRun::Yes;
  }
  ShouldRun::No
}

fn init(
  mut physics_manager: ResMut<PhysicsManager>,
  // game_res: Res<GameResource>,
) {
  println!("init()");
  let config = ChunkManager::default().config;
  // let config = game_res.chunk_manager.config.clone();
  physics_manager.load_terrains(&config);
}

fn process_input(
  mut commands: Commands,
  mut physics_res: ResMut<PhysicsResource>,
  thread_pool: Res<AsyncComputeTaskPool>,
) {
  if physics_res.events.len() > 0 {
    // println!("process_input() {}", physics_res.events[physics_res.events.len() - 1].tick );
    let events = physics_res.events.clone();
    let input = physics_res.input_sender.clone();
    let task = thread_pool.spawn(async move {
      let _ = input.send(Input {
        events: events
      }).await;
      InputTask
    });
    commands.spawn().insert(task);

    physics_res.events.clear();
  }
}

fn process_input_done(
  mut commands: Commands,
  mut tasks: Query<(Entity, &mut Task<InputTask>)>,
) {
  for (entity, mut task) in tasks.iter_mut() {
    if let Some(_r) = future::block_on(future::poll_once(&mut *task)) {
      commands.entity(entity).remove::<Task<InputTask>>();
      // println!("process_input_done");
    }
  }
}

fn start_simulator(
  mut commands: Commands, 
  thread_pool: Res<AsyncComputeTaskPool>,
  physics_manager: Res<PhysicsManager>,
  // game_res: Res<GameResource>,
  mut physics_res: ResMut<PhysicsResource>,
) {
  println!("start_simulator()");
  // let config = game_res.chunk_manager.config.clone();
  let config = ChunkManager::default().config;
  let mut input_rcv = physics_res.input_receiver.take().unwrap();
  let output = physics_res.output_sender.clone();

  let mut manager = physics_manager.clone();
  let task = thread_pool.spawn(async move {
    let mut physics = Physics::default();
    let mut frames = 0;
    loop {
      manager.sim_update(
        &mut physics,
        &config,
        &mut input_rcv,
        &output,
        &mut frames
      ).await;

      async_std::task::sleep(Duration::from_secs_f32(1.0 / 240.0)).await;
    }
  });
  commands.spawn().insert(task);
}

fn process_output(
  mut physics_res: ResMut<PhysicsResource>,
) {
  physics_res.outputs.clear();

  loop {
    match physics_res.output_receiver.as_mut().unwrap().try_recv() {
      Ok(output) => {
        // println!("output.players.len() {}", output.players.len());
        // for player in output.players.iter() {
        //   println!("output {:?}", player.pos);
        // }
        // println!("process_output tick {}", output.event.tick);
        physics_res.outputs.push_back(output.clone());
        // physics_res.output_tick = output.event.tick;
  
        // println!("output.data.len() {}", output.data.len());
        physics_res.output_tick = output.event.tick;
        physics_res.output_init = true;
      }
      Err(_e) => { break; } // FIXME: Handle error later
    }
  }
}

fn process_output_done(
  mut commands: Commands,
  mut tasks: Query<(Entity, &mut Task<Output>)>,
) {
  for (entity, mut task) in tasks.iter_mut() {
    if let Some(_r) = future::block_on(future::poll_once(&mut *task)) {
      commands.entity(entity).remove::<Task<Output>>();
      println!("done");
    }
  }
}

struct InputTask;

pub struct Input {
  pub events: VecDeque<Event>,
}

#[derive(Clone)]
pub struct Output {
  pub event: Event,
  pub players: Vec<Player>,
  // pub data: Vec<Data>,
}

impl Default for Output {
  fn default() -> Self {
    Self {
      event: Event::default(),
      players: Vec::new(),
      // data: Vec::new(),
    }
  }
}

pub struct PhysicsResource {
  // Should be private, have to change later
  pub input_sender: Sender<Input>,
  pub input_receiver: Option<Receiver<Input>>,
  pub output_sender: Sender<Output>,
  pub output_receiver: Option<Receiver<Output>>,
  // Should be private, have to change later

  pub events: VecDeque<Event>,
  pub saved_events: Vec<Event>,
  pub output: Option<Output>,
  pub prev_tick: u128,
  pub cur_tick: u128,
  pub output_tick: u128,
  pub output_init: bool,
  pub outputs: VecDeque<Output>,
  pub start: bool,
}

impl Default for PhysicsResource {
  fn default() -> Self {
    let channel_buffer = 100;
    let (input_sender, input_receiver) = mpsc::channel(channel_buffer);
    let (output_sender, output_receiver) = mpsc::channel(channel_buffer);
    Self {
      input_sender: input_sender,
      input_receiver: Some(input_receiver),
      output_sender: output_sender,
      output_receiver: Some(output_receiver),
      events: VecDeque::new(),
      saved_events: Vec::new(),
      output: None,
      prev_tick: 0,
      cur_tick: 0,
      output_tick: 0,
      output_init: false,
      outputs: VecDeque::new(),
      start: false,
    }
  }
}







