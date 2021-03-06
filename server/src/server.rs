use cgmath::{Aabb3, Point3};
use rand;
use std::collections::HashMap;
use std::sync::Mutex;
use time;

use common::communicate::{ServerToClient, ClientId};
use common::entity::EntityId;
use common::id_allocator::IdAllocator;
use common::interval_timer::IntervalTimer;
use common::lod::OwnerId;
use common::socket::SendSocket;

use init_mobs::init_mobs;
use mob;
use physics::Physics;
use player::Player;
use sun::Sun;
use terrain_loader::TerrainLoader;

const UPDATES_PER_SECOND: u64 = 30;
const SUN_TICK_NS: u64 = 1600000;

pub struct Client {
  pub socket: SendSocket,
}

impl Client {
  pub fn send(&mut self, msg: ServerToClient) {
    use bincode::SizeLimit;
    use bincode::rustc_serialize::encode;
    let msg = encode(&msg, SizeLimit::Infinite).unwrap();
    match self.socket.write(msg.as_ref()) {
      Ok(()) => {},
      Err(err) => warn!("Error sending to client: {:?}", err),
    }
  }
}

// TODO: Audit for s/Mutex/RwLock.
pub struct Server {
  pub players: Mutex<HashMap<EntityId, Player>>,
  pub mobs: Mutex<HashMap<EntityId, mob::Mob>>,

  pub id_allocator: Mutex<IdAllocator<EntityId>>,
  pub owner_allocator: Mutex<IdAllocator<OwnerId>>,
  pub client_allocator: Mutex<IdAllocator<ClientId>>,

  pub physics: Mutex<Physics>,
  pub terrain_loader: TerrainLoader,
  pub rng: Mutex<rand::StdRng>,

  pub clients: Mutex<HashMap<ClientId, Client>>,

  pub sun: Mutex<Sun>,
  pub update_timer: Mutex<IntervalTimer>,
}

impl Server {
  #[allow(missing_docs)]
  pub fn new() -> Server {
    let world_width: u32 = 1 << 11;
    let world_width = world_width as f32;
    let physics =
      Physics::new(
        Aabb3::new(
          Point3 { x: -world_width, y: -512.0, z: -world_width },
          Point3 { x: world_width, y: 512.0, z: world_width },
        )
      );

    let id_allocator = IdAllocator::new();
    let owner_allocator = Mutex::new(IdAllocator::new());

    let server = Server {
      players: Mutex::new(HashMap::new()),
      mobs: Mutex::new(HashMap::new()),

      id_allocator: Mutex::new(id_allocator),
      owner_allocator: owner_allocator,
      client_allocator: Mutex::new(IdAllocator::new()),

      physics: Mutex::new(physics),
      terrain_loader: TerrainLoader::new(),
      rng: {
        let seed = [0];
        let seed: &[usize] = &seed;
        Mutex::new(rand::SeedableRng::from_seed(seed))
      },

      clients: Mutex::new(HashMap::new()),
      sun: Mutex::new(Sun::new(SUN_TICK_NS)),

      update_timer: {
        let now = time::precise_time_ns();
        let nanoseconds_per_second = 1000000000;
        Mutex::new(
          IntervalTimer::new(nanoseconds_per_second / UPDATES_PER_SECOND, now)
        )
      }
    };

    init_mobs(&server);
    server
  }
}
