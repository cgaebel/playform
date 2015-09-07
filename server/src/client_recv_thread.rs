use cgmath::{Point, Point3, Vector3, Aabb3};
use std::convert::AsRef;
use std::f32::consts::PI;
use std::sync::mpsc::channel;
use std::time::Duration;
use thread_scoped;

use common::communicate::{ClientToServer, ServerToClient};
use common::entity;
use common::serialize;
use common::serialize::Copyable;
use common::socket::SendSocket;

use player::Player;
use server::{Client, Server};
use terrain;
use voxel;
use update_gaia::{ServerToGaia, LoadReason};

fn center(bounds: &Aabb3<f32>) -> Point3<f32> {
  bounds.min.add_v(&bounds.max.to_vec()).mul_s(1.0 / 2.0)
}

fn cast(
  server: &Server,
  player_id: entity::EntityId,
) -> Option<voxel::Bounds> {
  let ray;
  {
    let players = server.players.lock().unwrap();
    let player = players.get(&player_id).unwrap();
    ray = player.forward_ray();
  }

  let terrain_loader = server.terrain_loader.lock().unwrap();
  terrain_loader.terrain.voxels.cast_ray(
    &ray,
    &mut |bounds, voxel| {
      match voxel {
        &terrain::voxel::T::Volume(voxel::Material::Empty) => None,
        _ => Some(bounds),
      }
    }
  )
}

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
      let client_thread = unsafe {
        thread_scoped::scoped(move || {
          let mut socket = SendSocket::new(client_url.as_ref(), Some(Duration::from_secs(30)));
          while let Some(msg) = to_client_recv.recv().unwrap() {
            // TODO: Don't do this allocation on every packet!
            let msg = serialize::encode(&msg).unwrap();
            socket.write(msg.as_ref());
          }
        })
      };

      let client_id = server.client_allocator.lock().unwrap().allocate();
      to_client_send.send(Some(ServerToClient::LeaseId(Copyable(client_id)))).unwrap();

      let client =
        Client {
          sender: to_client_send,
          thread: client_thread,
        };
      server.clients.lock().unwrap().insert(client_id, client);
    },
    ClientToServer::Ping(Copyable(client_id)) => {
      server.clients.lock().unwrap()
        .get(&client_id)
        .unwrap()
        .sender
        .send(Some(ServerToClient::Ping(Copyable(()))))
        .unwrap();
    },
    ClientToServer::AddPlayer(Copyable(client_id)) => {
      let mut player =
        Player::new(
          server.id_allocator.lock().unwrap().allocate(),
          &server.owner_allocator,
        );

      // TODO: shift upward until outside terrain
      let min = Point3::new(0.0, 64.0, 4.0);
      let max = min.add_v(&Vector3::new(1.0, 2.0, 1.0));
      let bounds = Aabb3::new(min, max);
      server.physics.lock().unwrap().insert_misc(player.entity_id, bounds.clone());

      player.position = center(&bounds);
      player.rotate_lateral(PI / 2.0);

      let id = player.entity_id;
      let pos = player.position;

      server.players.lock().unwrap().insert(id, player);

      let clients = server.clients.lock().unwrap();
      let client = clients.get(&client_id).unwrap();
      client.sender.send(
        Some(ServerToClient::PlayerAdded(Copyable(id), Copyable(pos)))
      ).unwrap();
    },
    ClientToServer::StartJump(Copyable(player_id)) => {
      let mut players = server.players.lock().unwrap();
      let player = players.get_mut(&player_id).unwrap();
      if !player.is_jumping {
        player.is_jumping = true;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y + 0.3;
      }
    },
    ClientToServer::StopJump(Copyable(player_id)) => {
      let mut players = server.players.lock().unwrap();
      let player = players.get_mut(&player_id).unwrap();
      if player.is_jumping {
        player.is_jumping = false;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y - 0.3;
      }
    },
    ClientToServer::Walk(Copyable(player_id), Copyable(v)) => {
      let mut players = server.players.lock().unwrap();
      let mut player = players.get_mut(&player_id).unwrap();
      player.walk(v);
    },
    ClientToServer::RotatePlayer(Copyable(player_id), Copyable(v)) => {
      let mut players = server.players.lock().unwrap();
      let mut player = players.get_mut(&player_id).unwrap();
      player.rotate_lateral(v.x);
      player.rotate_vertical(v.y);
    },
    ClientToServer::RequestBlock(Copyable(client_id), Copyable(position), Copyable(lod)) => {
      update_gaia(ServerToGaia::Load(position, lod, LoadReason::ForClient(client_id)));
    },
    ClientToServer::Add(Copyable(player_id)) => {
      let bounds = cast(server, player_id);

      bounds.map(|bounds| {
        let (low, high) = bounds.corners();
        let mut bottom = low.add_v(&high.to_vec()).div_s(2.0);
        bottom.y = low.y;
        update_gaia(ServerToGaia::AddTree(bottom));
      });
    },
    ClientToServer::Remove(Copyable(player_id)) => {
      let bounds = cast(server, player_id);

      bounds.map(|bounds| {
        debug!("bounds {:?}", bounds);
        let center = bounds.center();
        let r = 8.0;
        let brush =
          voxel::field::sphere::T {
            center: center,
            radius: r,
            material: voxel::Material::Empty,
          };
        update_gaia(ServerToGaia::RemoveSphere(brush));
      });
    },
  };
}
