use std::sync::Mutex;
use stopwatch;
use time;

use common::block_position::BlockPosition;
use common::communicate::{ClientToServer, ServerToClient, TerrainBlockSend};
use common::serialize::Copyable;
use common::surroundings_loader::LODChange;

use client::Client;
use load_terrain::{load_terrain_block, lod_index};
use server_update::apply_server_update;
use view_update::ClientToView;

pub fn update_thread<RecvServer, RecvBlock, UpdateView, UpdateServer, QueueBlock>(
  quit: &Mutex<bool>,
  client: &Client,
  recv_server: &mut RecvServer,
  recv_block: &mut RecvBlock,
  update_view: &mut UpdateView,
  update_server: &mut UpdateServer,
  queue_block: &mut QueueBlock,
) where
  RecvServer: FnMut() -> Option<ServerToClient>,
  RecvBlock: FnMut() -> Option<TerrainBlockSend>,
  UpdateView: FnMut(ClientToView),
  UpdateServer: FnMut(ClientToServer),
  QueueBlock: FnMut(TerrainBlockSend),
{
  'update_loop: loop {
    if *quit.lock().unwrap() == true {
      break 'update_loop;
    } else {
      stopwatch::time("update_iteration", || {
        if let Some(up) = recv_server() {
          let start = time::precise_time_ns();
          trace!("apply server update, {:?}", start);
          apply_server_update(
            client,
            update_view,
            update_server,
            queue_block,
            up,
          );
          let end = time::precise_time_ns();
          trace!("done apply server update, {:?}us", (end - start) / 1000);
        } else {
          let mut updated_surroundings = false;

          let start = time::precise_time_ns();
          trace!("update surroundings, {:?}", start);
          stopwatch::time("update_surroundings", || {
            let position = *client.player_position.lock().unwrap();
            let position = BlockPosition::from_world_position(&position);
            let mut loaded_blocks = client.loaded_blocks.lock().unwrap();
            let mut surroundings_loader = client.surroundings_loader.lock().unwrap();
            for lod_change in surroundings_loader.updates(position).take(1 << 16) {
              updated_surroundings = true;
              match lod_change {
                LODChange::Load(block_position, distance) => {
                  trace!("load {:?}", block_position);
                  stopwatch::time("update_surroundings.load", || {
                    let lod = stopwatch::time("update_surroundings.load.lod_index", || { lod_index(distance) });
                    let loaded_lod =
                      stopwatch::time("update_surroundings.load.loaded_lod", || {
                        loaded_blocks
                        .get(&block_position)
                        .map(|&(_, lod)| lod)
                      });
                    if loaded_lod != Some(lod) {
                      stopwatch::time("update_surroundings.load.update_server", || {
                        update_server(
                          ClientToServer::RequestBlock(
                            Copyable(client.id),
                            Copyable(block_position),
                            Copyable(lod),
                          )
                        );
                      });
                    } else {
                      debug!("Not re-loading {:?} at {:?}", block_position, lod);
                    }
                  })
                },
                LODChange::Unload(block_position) => {
                  trace!("unload {:?}", block_position);
                  stopwatch::time("update_surroundings.unload", || {
                    // The block removal code is duplicated elsewhere.

                    loaded_blocks
                      .remove(&block_position)
                      // If it wasn't loaded, don't unload anything.
                      .map(|(block, prev_lod)| {
                        for id in &block.ids {
                          update_view(ClientToView::RemoveTerrain(*id));
                        }
                        update_view(ClientToView::RemoveBlockData(block_position, prev_lod));
                      });
                  })
                },
              };
            }
          });

          let end = time::precise_time_ns();
          trace!("done updating surroundings, {:?}us", (end - start) / 1000);

          if !updated_surroundings {
            if let Some(block) = recv_block() {
              trace!("Got block: {:?} at {:?}", block.position, block.lod);
              let start = time::precise_time_ns();
              trace!("load terrain block, {:?}", start);
              stopwatch::time("load_terrain_block", || {
                load_terrain_block(
                  client,
                  update_view,
                  block,
                );
              });
              let end = time::precise_time_ns();
              trace!("done load terrain block, {:?}us", (end - start) / 1000);
            }
          }
        }
      });
    }
  }
}
