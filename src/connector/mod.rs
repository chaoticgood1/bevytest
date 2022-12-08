use bevy::{prelude::*, ecs::system::Resource};
use std::fmt::Display;

use ironverse::{FromNet, ToNet, Topic, types::{SwarmEvent, GossipsubEvent}, Multiaddr};
use serde::{Deserialize, Serialize};
use std::env;

pub mod net;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ConnectionState {
  Idle,
  Startup,
  Connected,
  OnLobby,
}

pub struct CustomPlugin;
impl Plugin for CustomPlugin {
    fn build(&self, app: &mut App) {
      let private_keys = vec![
        [8, 1, 18, 64, 143, 217, 138, 187, 8, 122, 58, 224, 61, 123, 184, 159, 142, 154, 50, 28, 118, 136, 81, 182, 0, 62, 208, 2, 30, 163, 98, 200, 101, 42, 197, 37, 75, 167, 43, 113, 218, 98, 89, 78, 25, 233, 206, 56, 135, 218, 174, 168, 66, 207, 202, 213, 80, 92, 144, 131, 79, 85, 64, 9, 111, 23, 90, 201],
        [8, 1, 18, 64, 20, 44, 185, 177, 32, 46, 106, 102, 131, 232, 223, 27, 172, 43, 7, 232, 76, 140, 46, 31, 117, 201, 179, 209, 126, 168, 163, 74, 65, 22, 248, 252, 68, 160, 200, 43, 70, 200, 124, 247, 238, 136, 34, 74, 164, 210, 176, 251, 89, 191, 239, 68, 136, 103, 106, 163, 177, 95, 190, 218, 92, 194, 146, 209],
        [8, 1, 18, 64, 124, 32, 193, 168, 134, 21, 66, 113, 163, 211, 211, 22, 26, 7, 182, 29, 13, 187, 2, 202, 243, 156, 105, 163, 77, 180, 115, 145, 106, 59, 171, 56, 54, 252, 21, 64, 167, 238, 104, 197, 48, 229, 31, 193, 151, 154, 52, 119, 184, 132, 99, 177, 240, 193, 3, 171, 24, 54, 97, 254, 11, 165, 116, 240],
      ];
      let args: Vec<String> = env::args().collect();
      if args.len() == 0 {
        panic!("Please include ip address to connect in cmd arguments");
      }
      let key_index = args[2].parse::<usize>().unwrap();
      let key = private_keys[key_index].clone();

      app
        .insert_resource(ConnectorResource::default())
        .add_state(ConnectionState::Startup)
        .add_event::<IncomingEvent>()
        .add_event::<OutgoingEvent>()
        .add_event::<Incoming>()
        .add_event::<Outgoing>()
        .add_plugin(net::CustomPlugin::<IncomingEvent, OutgoingEvent>::new(key))
        .add_system_to_stage(
          CoreStage::PreUpdate,
          receiver::<IncomingEvent>
        )
        .add_system_set_to_stage(
          CoreStage::PostUpdate,
          SystemSet::new()
            .with_system(sender::<OutgoingEvent>)
        );
    }
}

fn receiver<I>(
  mut event_reader: EventReader<I>,
  mut conn_res: ResMut<ConnectorResource>,
  
) where
I: Resource + Clone + Into<Incoming> {
  conn_res.incomings.clear();
  for event in event_reader.iter() {
    let incoming: Incoming = event.clone().into();

    // println!("incoming {:?} {:?} {:?}", incoming.sender, incoming.channel, incoming.msg);
    conn_res.incomings.push(incoming);
  }
}

fn sender<O>(
  mut event_writer: EventWriter<O>,
  mut conn_res: ResMut<ConnectorResource>,
) where
O: Resource + From<Outgoing> {
  // if conn_res.outgoings.len() > 0 {
  //   let outgoing = conn_res.outgoings.remove(0);
  //   let _ = event_writer.send(outgoing.clone().into());
  // }


  for o in conn_res.outgoings.iter() {
    let _ = event_writer.send(o.clone().into());
  }
  conn_res.outgoings.clear();
}


pub struct ConnectorResource {
  pub incomings: Vec<Incoming>,
  pub outgoings: Vec<Outgoing>,
}

impl Default for ConnectorResource {
  fn default() -> Self {
    Self {
      incomings: Vec::new(),
      outgoings: Vec::new(),
    }
  }
}

#[derive(Clone)]
pub struct IncomingEvent(FromNet);
impl From<FromNet> for IncomingEvent {
    fn from(from_net: FromNet) -> Self { Self(from_net) }
}
impl From<IncomingEvent> for Incoming {
  fn from(incoming: IncomingEvent) -> Self { 
    match incoming.0 {
      FromNet::None => Self::default(),
      FromNet::SwarmEvent(event) => {
        match event {
          SwarmEvent::NewListenAddr { listener_id: _, address } => {
            Self{
              channel: "Info".into(), 
              sender: "NewListenAddr".into(), 
              msg: ron::to_string(&NewListenAddr {
                address: address.to_string()
              }).unwrap()
            }
          },
          SwarmEvent::ConnectionEstablished { 
            peer_id, 
            endpoint: _, 
            num_established, 
            concurrent_dial_errors: _ 
          } => {
            Self{
              channel: "Info".into(), 
              sender: "ConnectionEstablished".into(), 
              msg: ron::to_string(&ConnectionEstablished {
                peer_id: peer_id.to_base58(),
                num_established: u32::from(num_established)
              }).unwrap()
            }
          }
          SwarmEvent::Behaviour(swarm_event) => {
            match swarm_event {
              GossipsubEvent::Message { message, .. } => {
                let mut source = String::from("unknown");
                if let Some(src) = message.source {
                    source = src.to_string();
                }
                let msg = String::from_utf8_lossy(&message.data).to_string();
                Self{channel: message.topic.to_string(), sender: source, msg: format!("{}", msg)}
              },
              GossipsubEvent::Subscribed { peer_id, topic } => {
                Self{
                  channel: "Info".into(), 
                  sender: "Subscribed".into(), 
                  msg: ron::to_string(&Subscribe {
                    peer_id: peer_id.to_base58(),
                    topic: topic.into_string()
                  }).unwrap()
                }
              }
              _ => Self{channel: "Info".into(), sender: "Network".into(), msg: format!("{:?}", swarm_event)},
            }
          },
          _ => Self{channel: "Info".into(), sender: "Network".into(), msg: format!("{:?}", event)},
        }
      },
      _ => Self{channel: "Error".into(), sender: "Network".into(), msg: format!("{:?}", incoming.0)},
    }
  }
}

#[derive(Clone)]
pub struct OutgoingEvent(ToNet);
impl From<Outgoing> for OutgoingEvent {
  fn from(outgoing: Outgoing) -> Self { 
    match outgoing.cmd.as_str() {
      "dial" => Self(ToNet::Dial { address: outgoing.msg.split(" ").last().unwrap_or("").parse().unwrap_or(Multiaddr::empty()) }),
      "sub" => Self(ToNet::Sub { topic: Topic::new(outgoing.channel) }),
      "unsub" => Self(ToNet::Unsub { topic: Topic::new(outgoing.channel) }),
      "" => Self(ToNet::Pub { topic: Topic::new(outgoing.channel), data: outgoing.msg.into() }),
      _ => Self(ToNet::None)
    }
  }
}
impl From<OutgoingEvent> for ToNet {
  fn from(outgoing: OutgoingEvent) -> Self { outgoing.0 }
}


#[derive(Clone, Default)]
pub struct Outgoing{
    pub channel: String,
    pub cmd: String,
    pub msg: String
}
#[derive(Clone, Default)]
pub struct Incoming{
    pub channel: String,
    pub sender: String,
    pub msg: String,
}
impl Display for Incoming {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}: {})", self.sender, self.msg)
  }
}


#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct NewListenAddr {
  pub address: String
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Subscribe {
  pub peer_id: String,
  pub topic: String,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionEstablished {
  pub peer_id: String,
  pub num_established: u32,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Message {
  pub subscribe: Subscribe,
  pub chat: Chat,
  // pub player_event: PlayerEvent
}


#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Chat {
  pub text: String,
}
