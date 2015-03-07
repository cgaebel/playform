use rustc_serialize::json;
use std::sync::mpsc::channel;
use std::thread;

use common::communicate::{ClientToServer, ServerToClient};
use common::socket::SendSocket;

use server::{Client, Server};
use update_gaia::{ServerToGaia, LoadReason};

#[inline]
pub fn apply_client_update<UpdateGaia>(
  server: &Server,
  update_gaia: &mut UpdateGaia,
  update: ClientToServer,
) where
  UpdateGaia: FnMut(ServerToGaia),
{
  match update {
    ClientToServer::Init(client_url) => {
      info!("Sending to {}.", client_url);

      let (to_client_send, to_client_recv) = channel();
      let client_thread = {
        thread::scoped(move || {
          let mut socket = SendSocket::new(client_url.as_slice());
          while let Some(msg) = to_client_recv.recv().unwrap() {
            // TODO: Don't do this allocation on every packet!
            let msg = json::encode(&msg).unwrap();
            socket.write(msg.as_bytes());
          }
        })
      };

      let client_id = server.client_allocator.lock().unwrap().allocate();
      to_client_send.send(Some(ServerToClient::LeaseId(client_id))).unwrap();

      server.inform_client(
        &mut |msg| { to_client_send.send(Some(msg)).unwrap() },
      );

      let client =
        Client {
          sender: to_client_send,
          thread: client_thread,
        };
      server.clients.lock().unwrap().insert(client_id, client);
    },
    ClientToServer::StartJump => {
      let mut player = server.player.lock().unwrap();
      if !player.is_jumping {
        player.is_jumping = true;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y + 0.3;
      }
    },
    ClientToServer::StopJump => {
      let mut player = server.player.lock().unwrap();
      if player.is_jumping {
        player.is_jumping = false;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y - 0.3;
      }
    },
    ClientToServer::Walk(v) => {
      let mut player = server.player.lock().unwrap();
      player.walk(v);
    },
    ClientToServer::RotatePlayer(v) => {
      let mut player = server.player.lock().unwrap();
      player.rotate_lateral(v.x);
      player.rotate_vertical(v.y);
    },
    ClientToServer::RequestBlock(position, lod) => {
      update_gaia(ServerToGaia::Load(position, lod, LoadReason::ForClient));
    },
  };
}
