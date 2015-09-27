/// Creator of the earth.

use std::cmp::Ordering;
use std::ops::DerefMut;
use stopwatch;

use common::communicate::{ClientId, ServerToClient, TerrainBlockSend};
use common::lod::{LODIndex, OwnerId};
use common::serialize::Copyable;
use common::block_position::BlockPosition;

use server::Server;
use terrain;
use terrain_loader::TerrainLoader;
use voxel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadReason {
  Local(OwnerId),
  ForClient(ClientId, u16),
}

impl Ord for LoadReason {
  fn cmp(&self, other: &LoadReason) -> Ordering {
    match (self, other) {
      (&LoadReason::Local(_), &LoadReason::Local(_)) => Ordering::Equal,
      (&LoadReason::ForClient(_, _), &LoadReason::Local(_)) => Ordering::Less,
      (&LoadReason::Local(_), &LoadReason::ForClient(_, _)) => Ordering::Greater,
      (&LoadReason::ForClient(_, p1), &LoadReason::ForClient(_, p2)) =>
        // Compare in reverse order because smaller client priorities should be closer to the top.
        p2.cmp(&p1)
    }
  }
}

impl PartialOrd for LoadReason {
  fn partial_cmp(&self, other: &LoadReason) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerToGaiaCmp {
  Load(LoadReason),
  Brush,
}

pub enum ServerToGaia {
  Load(BlockPosition, LODIndex, LoadReason),
  Brush(voxel::brush::T<Box<voxel::mosaic::T<Material=terrain::voxel::Material> + Send>>),
}

impl ServerToGaia {
  pub fn to_cmp(&self) -> ServerToGaiaCmp {
    match self {
      &ServerToGaia::Load(_, _, x) => ServerToGaiaCmp::Load(x),
      &ServerToGaia::Brush(_) => ServerToGaiaCmp::Brush,
    }
  }
}

impl PartialEq for ServerToGaia { fn eq(&self, other: &Self) -> bool { self.to_cmp().eq(&other.to_cmp()) } }
impl Eq for ServerToGaia {}
impl PartialOrd for ServerToGaia { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.to_cmp().partial_cmp(&other.to_cmp()) } }
impl Ord for ServerToGaia { fn cmp(&self, other: &Self) -> Ordering { self.to_cmp().cmp(&other.to_cmp()) } }

// TODO: Consider adding terrain loads to a thread pool instead of having one monolithic separate thread.
pub fn update_gaia(
  server: &Server,
  update: ServerToGaia,
) {
  stopwatch::time("update_gaia", move || {
    match update {
      ServerToGaia::Load(position, lod, load_reason) => {
        stopwatch::time("terrain.load", || {
          // TODO: Just lock `terrain` for the check and then the move;
          // don't lock for the whole time where we're generating the block.
          let mut terrain_loader = server.terrain_loader.lock().unwrap();
          let terrain_loader = terrain_loader.deref_mut();
          let lod_map = &mut terrain_loader.lod_map;
          let in_progress_terrain = &mut terrain_loader.in_progress_terrain;
          let block =
            terrain_loader.terrain.load(
              &server.id_allocator,
              &position,
              lod,
            );

          match load_reason {
            LoadReason::Local(owner) => {
              // TODO: Check that this block isn't stale, i.e. should still be loaded.
              // Maybe this should just ping the original thread, same as we ping the client.
              TerrainLoader::insert_block(
                block,
                &position,
                lod,
                owner,
                &server.physics,
                lod_map,
                in_progress_terrain,
              );
            },
            LoadReason::ForClient(id, _) => {
              let mut clients = server.clients.lock().unwrap();
              let client = clients.get_mut(&id).unwrap();
              client.send(
                ServerToClient::UpdateBlock(TerrainBlockSend {
                  position: Copyable(position),
                  block: block.clone(),
                  lod: Copyable(lod),
                })
              );
            },
          }
        });
      },
      ServerToGaia::Brush(brush) => {
        let mut terrain_loader = server.terrain_loader.lock().unwrap();
        terrain_loader.terrain.brush(
          &server.id_allocator,
          &brush,
          |block, position, lod| {
            let mut clients = server.clients.lock().unwrap();
            for (_, client) in clients.iter_mut() {
              client.send(
                ServerToClient::UpdateBlock(TerrainBlockSend {
                  position: Copyable(*position),
                  block: block.clone(),
                  lod: Copyable(lod),
                })
              );
            }
          },
        );
      },
    };
  })
}
