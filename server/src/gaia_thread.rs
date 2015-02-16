/// Creator of the earth.

use common::id_allocator::IdAllocator;
use common::process_events::process_channel;
use common::stopwatch::TimerSet;
use common::terrain_block::{BLOCK_WIDTH, TEXTURE_WIDTH};
use gaia_update::ServerToGaia;
use opencl_context::CL;
use server_update::GaiaToServer;
use std::old_io::timer;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::time::duration::Duration;
use terrain::terrain::Terrain;
use terrain::texture_generator::TerrainTextureGenerator;

pub fn gaia_thread(
  ups_from_server: Receiver<ServerToGaia>,
  ups_to_server: Sender<GaiaToServer>,
  terrain: Arc<Mutex<Terrain>>,
) {
  let ups_from_server = &ups_from_server;
  let ups_to_server = &ups_to_server;

  let id_allocator = Mutex::new(IdAllocator::new());
  let id_allocator = &id_allocator;
  let timers = TimerSet::new();

  let cl = unsafe {
    CL::new()
  };

  let texture_generators = [
    TerrainTextureGenerator::new(&cl, TEXTURE_WIDTH[0], BLOCK_WIDTH as u32),
    TerrainTextureGenerator::new(&cl, TEXTURE_WIDTH[1], BLOCK_WIDTH as u32),
    TerrainTextureGenerator::new(&cl, TEXTURE_WIDTH[2], BLOCK_WIDTH as u32),
    TerrainTextureGenerator::new(&cl, TEXTURE_WIDTH[3], BLOCK_WIDTH as u32),
  ];

  loop {
    let quit =
      !process_channel(
        ups_from_server,
        |update|
          update.apply(
            &timers,
            &cl,
            id_allocator,
            terrain.clone(),
            &texture_generators,
            ups_to_server,
          )
      );
    if quit {
      break;
    }

    timer::sleep(Duration::milliseconds(0));
  }
}
