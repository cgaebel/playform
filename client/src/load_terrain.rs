use num;
use std::collections::hash_map::Entry::{Vacant, Occupied};

use common::block_position::BlockPosition;
use common::communicate::TerrainBlockSend;
use common::lod::LODIndex;
use common::serialize;
use common::surroundings_loader::radius_between;

use client::{Client, LOD_THRESHOLDS};
use view_update::ClientToView;

pub fn load_terrain_block<UpdateView>(
  client: &Client,
  update_view: &mut UpdateView,
  block: TerrainBlockSend,
) where
  UpdateView: FnMut(ClientToView),
{
  let player_position =
    BlockPosition::from_world_position(&client.player_position.lock().unwrap().clone());
  let distance = radius_between(&player_position, &block.position.0);

  if distance > client.max_load_distance {
    debug!(
      "Not loading {:?}: too far away from player at {:?}.",
      block.position,
      player_position,
    );
    return;
  }

  let lod = lod_index(distance);
  if lod != block.lod.0 {
    debug!(
      "Not loading {:?}: given LOD {:?} is not the desired LOD {:?}.",
      block.position,
      block.lod,
      lod
    );
    return;
  }

  let block_data = serialize::lazy::force(&block.block);

  if !block_data.ids.is_empty() {
    update_view(ClientToView::AddBlock(block.position.0, block_data.clone(), block.lod.0));
  }

  match client.loaded_blocks.lock().unwrap().entry(block.position.0) {
    Vacant(entry) => {
      entry.insert((block_data, block.lod.0));
    },
    Occupied(mut entry) => {
      {
        // The block removal code is duplicated elsewhere.

        let &(ref prev_block, prev_lod) = entry.get();
        for &id in &prev_block.ids {
          update_view(ClientToView::RemoveTerrain(id));
        }
        update_view(ClientToView::RemoveBlockData(block.position.0, prev_lod));
      }
      entry.insert((block_data, block.lod.0));
    },
  };
}

pub fn lod_index(distance: i32) -> LODIndex {
  assert!(distance >= 0);
  let mut lod = 0;
  while
    lod < LOD_THRESHOLDS.len()
    && LOD_THRESHOLDS[lod] < distance
  {
    lod += 1;
  }
  LODIndex(num::traits::FromPrimitive::from_usize(lod).unwrap())
}
