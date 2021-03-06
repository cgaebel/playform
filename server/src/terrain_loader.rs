use std::sync::Mutex;
use stopwatch;

use common::block_position::BlockPosition;
use common::entity::EntityId;
use common::id_allocator::IdAllocator;
use common::lod::{LOD, LODIndex, OwnerId, LODMap};
use common::terrain_block::TerrainBlock;

use in_progress_terrain::InProgressTerrain;
use physics::Physics;
use terrain::{Terrain, Seed};
use update_gaia;
use update_gaia::LoadReason;

// TODO: Consider factoring this logic such that what to load is separated from how it's loaded.

/// Load and unload TerrainBlocks from the game.
/// Each TerrainBlock can be owned by a set of owners, each of which can independently request LODs.
/// The maximum LOD requested is the one that is actually loaded.
pub struct TerrainLoader {
  pub terrain: Terrain,
  pub in_progress_terrain: Mutex<InProgressTerrain>,
  pub lod_map: Mutex<LODMap>,
}

impl TerrainLoader {
  pub fn new() -> TerrainLoader {
    TerrainLoader {
      terrain: Terrain::new(Seed::new(0)),
      in_progress_terrain: Mutex::new(InProgressTerrain::new()),
      lod_map: Mutex::new(LODMap::new()),
    }
  }

  // TODO: Avoid the double-lookup when unload and load the same index.

  pub fn load<LoadBlock>(
    &self,
    id_allocator: &Mutex<IdAllocator<EntityId>>,
    physics: &Mutex<Physics>,
    block_position: &BlockPosition,
    new_lod: LOD,
    owner: OwnerId,
    load_block: &mut LoadBlock,
  ) where LoadBlock: FnMut(update_gaia::Message)
  {
    let prev_lod;
    let max_lod_changed: bool;
    let mut lod_map = self.lod_map.lock().unwrap();
    let mut in_progress_terrain = self.in_progress_terrain.lock().unwrap();
    match lod_map.get(block_position, owner) {
      Some((Some(prev), lods)) => {
        prev_lod = Some(prev);
        if new_lod == prev {
          return;
        }

        if new_lod < prev {
          max_lod_changed = lods.iter().filter(|&&(_, l)| l >= prev).count() < 2;
        } else {
          max_lod_changed = lods.iter().filter(|&&(_, l)| l >= new_lod).count() == 0;
        }
      },
      Some((None, lods)) => {
        max_lod_changed = lods.iter().filter(|&&(_, l)| l >= new_lod).count() == 0;
        prev_lod = None;
      },
      None => {
        max_lod_changed = true;
        prev_lod = None;
      },
    }

    if !max_lod_changed {
      // Maximum LOD is unchanged.
      let (_, change) = lod_map.insert(*block_position, new_lod, owner);
      assert!(change.is_none());
      return;
    }

    match new_lod {
      LOD::Placeholder => {
        let (_, change) = lod_map.insert(*block_position, new_lod, owner);
        let change = change.unwrap();
        assert!(change.loaded == None);
        assert!(prev_lod == None);
        assert!(change.desired == Some(LOD::Placeholder));
        in_progress_terrain.insert(id_allocator, physics, block_position);
      },
      LOD::LodIndex(new_lod) => {
        let mut generate_block = || {
          debug!("{:?} requested from gaia", block_position);
          load_block(
            update_gaia::Message::Load(*block_position, new_lod, LoadReason::Local(owner))
          );
        };
        match self.terrain.all_blocks.lock().unwrap().get(block_position) {
          None => {
            generate_block();
          },
          Some(mipmesh) => {
            match mipmesh.lods[new_lod.0 as usize].as_ref() {
              None => {
                generate_block();
              },
              Some(block) => {
                TerrainLoader::insert_block(
                  block,
                  block_position,
                  new_lod,
                  owner,
                  physics,
                  &mut *lod_map,
                  &mut *in_progress_terrain,
                );
              },
            }
          }
        };
      },
    };
  }

  pub fn insert_block(
    block: &TerrainBlock,
    position: &BlockPosition,
    lod: LODIndex,
    owner: OwnerId,
    physics: &Mutex<Physics>,
    lod_map: &mut LODMap,
    in_progress_terrain: &mut InProgressTerrain,
  ) {
    let lod = LOD::LodIndex(lod);
    let (_, change) = lod_map.insert(*position, lod, owner);
    // TODO: This should be an unwrap, but the preconditions of another TODO aren't
    // satisfied in src/update_gaia.rs.
    // (i.e. blocks sometimes get here when they're stale).
    let change = match change {
      None => return,
      Some(change) => change,
    };
    assert!(change.desired == Some(lod));
    change.loaded.map(|loaded_lod|
      match loaded_lod {
        LOD::Placeholder => {
          in_progress_terrain.remove(physics, position);
        }
        LOD::LodIndex(_) => {
          stopwatch::time("terrain_loader.load.unload", || {
            let mut physics = physics.lock().unwrap();
            for id in &block.ids {
              physics.remove_terrain(*id);
            }
          });
        },
      }
    );

    stopwatch::time("terrain_loader.load.physics", || {
      let mut physics = physics.lock().unwrap();
      for &(ref id, ref bounds) in &block.bounds {
        physics.insert_terrain(*id, bounds.clone());
      }
    });
  }

  pub fn unload(
    &self,
    physics: &Mutex<Physics>,
    block_position: &BlockPosition,
    owner: OwnerId,
  ) {
    let (_, mlod_change) =
      self.lod_map.lock().unwrap().remove(*block_position, owner);

    let lod_change;
    match mlod_change {
      None => {
        return;
      },
      Some(c) => lod_change = c,
    }

    lod_change.loaded.map(|loaded_lod| {
      match loaded_lod {
        LOD::Placeholder => {
          self.in_progress_terrain.lock().unwrap().remove(physics, block_position);
        }
        LOD::LodIndex(loaded_lod) => {
          stopwatch::time("terrain_loader.unload", || {
            match self.terrain.all_blocks.lock().unwrap().get(block_position) {
              None => {
                // Unloaded before the load request completed.
              },
              Some(block) => {
                if let Some(&Some(ref block)) = block.lods.get(loaded_lod.0 as usize) {
                  let mut physics = physics.lock().unwrap();
                  for id in &block.ids {
                    physics.remove_terrain(*id);
                  }
                } else {
                  // Unloaded before the load request completed.
                }
              },
            }
          });
        },
      }
    });
  }
}
