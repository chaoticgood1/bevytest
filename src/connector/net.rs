use std::marker::PhantomData;
use bevy::{
  prelude::*, 
  ecs::system::Resource, 
  tasks::{Task, AsyncComputeTaskPool}
};
use ironverse::libp2p::identity::Keypair;
use ironverse::{
  FromNet, ToNet, Config,
  tokio::sync::mpsc,
  tokio::sync::mpsc::error,
  libp2p::{
    identity, PeerId,
  }
};

pub struct CustomPlugin<F, T> {
  from_net_type: PhantomData<F>,
  to_net_type: PhantomData<T>,
  private_key: [u8; 68],
}

impl<F, T> CustomPlugin<F, T> {
  pub fn new(private_key: [u8; 68]) -> Self {
    CustomPlugin { 
      from_net_type: PhantomData::<F>, 
      to_net_type: PhantomData::<T>,
      private_key: private_key,
    }
  }
}



impl<F, T> Plugin for CustomPlugin<F, T> where
F: Resource + From<FromNet>,
T: Resource + Clone + Into<ToNet> {
  fn build(&self, app: &mut App) {
    let local_key = identity::Keypair::from_protobuf_encoding(&self.private_key).unwrap();
    app
      .insert_resource(NetResource::new(10, local_key))
      .add_startup_system(startup)
      .add_system(network::<F, T>);
  }
}

fn startup(
  mut local: ResMut<NetResource>, 
  thread_pool: Res<AsyncComputeTaskPool>, 
) {
  if local.task.is_none() {
    let config = local.config.take().unwrap();
    local.task = Some(thread_pool.spawn(async move {
      ironverse::start_swarm(config).await.unwrap();
    }));
  }
}

fn network<F, T>(
  mut local: ResMut<NetResource>,
  mut event_reader: EventReader<T>, 
  mut event_writer: EventWriter<F>
) where
F: Resource + From<FromNet>, 
T: Resource + Clone + Into<ToNet> {
  loop {
    match local.from_net_receiver.try_recv() {
      Ok(event) => {
        event_writer.send(event.into());
      }
      Err(e) => {
        match e {
          error::TryRecvError::Empty => {},
          _ => event_writer.send(FromNet::TryRecvError(e.to_string()).into())
        }
        break;
      }
    }
  }
  
  for event in event_reader.iter() {
    let msg: ToNet = event.clone().into();
    if let Err(e) = local.to_net_sender.blocking_send(msg) {
      event_writer.send(FromNet::SendError(e.to_string()).into())
    }
  }
}

pub struct NetResource {
  pub to_net_sender: mpsc::Sender<ToNet>,
  pub from_net_receiver: mpsc::Receiver<FromNet>,
  pub task: Option<Task<()>>,
  pub local_peer_id: PeerId,
  pub config: Option<Config>,
  
}
impl NetResource {
  pub fn new(channel_buffer: usize, local_key: Keypair) -> Self {
    let local_peer_id = PeerId::from(local_key.public());
    println!("local_peer_id {:?}", local_peer_id);

    let (to_net_sender, to_net_receiver) = mpsc::channel(channel_buffer);
    let (from_net_sender, from_net_receiver) = mpsc::channel(channel_buffer);
    Self{
      to_net_sender,
      from_net_receiver,
      task: None,
      config: Some(Config{
        from_net_sender,
        to_net_receiver,
        local_key,
        local_peer_id,
      }),
      local_peer_id: local_peer_id
    }
  }
}

// Test the network code separate from using bevy
