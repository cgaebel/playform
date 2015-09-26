use cgmath::{Point, Point3, Vector, Vector3, Aabb3};
use rand;
use rand::distributions::IndependentSample;
use std::convert::AsRef;
use std::f32::consts::PI;
use std::ops::DerefMut;
use std::time::Duration;
use stopwatch;

use common::communicate::{ClientToServer, ServerToClient};
use common::entity;
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

pub fn apply_client_update<UpdateGaia>(
  server: &Server,
  update_gaia: &mut UpdateGaia,
  update: ClientToServer,
) where
  UpdateGaia: FnMut(ServerToGaia),
{
  stopwatch::time("apply_client_update", move || {
    match update {
      ClientToServer::Init(client_url) => {
        info!("Sending to {}.", client_url);

        let mut client =
          Client {
            socket: SendSocket::new(client_url.as_ref(), Some(Duration::from_secs(30))),
          };

        let client_id = server.client_allocator.lock().unwrap().allocate();
        client.send(ServerToClient::LeaseId(Copyable(client_id)));

        server.clients.lock().unwrap().insert(client_id, client);
      },
      ClientToServer::Ping(Copyable(client_id)) => {
        server.clients.lock().unwrap()
          .get_mut(&client_id)
          .unwrap()
          .send(ServerToClient::Ping(Copyable(())));
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

        let mut clients = server.clients.lock().unwrap();
        let client = clients.get_mut(&client_id).unwrap();
        client.send(
          ServerToClient::PlayerAdded(Copyable(id), Copyable(pos))
        );
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
      ClientToServer::RequestBlock(Copyable(client_id), Copyable(position), Copyable(lod), Copyable(priority)) => {
        update_gaia(ServerToGaia::Load(position, lod, LoadReason::ForClient(client_id, priority)));
      },
      ClientToServer::Add(Copyable(player_id)) => {
        let bounds = cast(server, player_id);

        bounds.map(|bounds| {
          let mut rng = server.rng.lock().unwrap();
          let rng = rng.deref_mut();

          let trunk_radius =
            rand::distributions::normal::Normal::new(2.0, 0.5)
            .ind_sample(rng);
          let trunk_radius =
            f64::max(1.0, f64::min(3.0, trunk_radius));

          let trunk_height =
            rand::distributions::normal::Normal::new(8.0 * trunk_radius, 2.0 * trunk_radius)
            .ind_sample(rng);
          let trunk_height =
            f64::max(4.0 * trunk_radius, f64::min(12.0 * trunk_radius, trunk_height));

          let leaf_radius =
            rand::distributions::normal::Normal::new(4.0 * trunk_radius, trunk_radius)
            .ind_sample(rng);
          let leaf_radius =
            f64::max(2.0 * trunk_radius, f64::min(6.0 * trunk_radius, leaf_radius));

          let (low, high) = bounds.corners();
          let mut bottom = low.add_v(&high.to_vec()).div_s(2.0);
          bottom.y = low.y;

          let trunk_height = trunk_height as f32;
          let trunk_radius = trunk_radius as f32;
          let leaf_radius = leaf_radius as f32;

          let tree =
            voxel::mosaic::translation::T {
              translation: bottom.to_vec(),
              mosaic: voxel::mosaic::tree::new(rng, trunk_height, trunk_radius, leaf_radius),
            };

          let center =
            bottom.add_v(&Vector3::new(0.0, trunk_height / 2.0, 0.0));
          let r = trunk_height / 2.0 + leaf_radius + 20.0;
          let brush =
            voxel::brush::T {
              bounds:
                Aabb3::new(
                  {
                    let low = center.add_v(&-Vector3::new(r, r, r));
                    Point3::new(low.x.floor() as i32, low.y.floor() as i32, low.z.floor() as i32)
                  },
                  {
                    let high = center.add_v(&Vector3::new(r, r, r));
                    Point3::new(high.x.ceil() as i32, high.y.ceil() as i32, high.z.ceil() as i32)
                  },
                ),
              mosaic: Box::new(tree) as Box<voxel::mosaic::T + Send>,
            };

          update_gaia(ServerToGaia::Brush(brush));
        });
      },
      ClientToServer::Remove(Copyable(player_id)) => {
        let bounds = cast(server, player_id);

        bounds.map(|bounds| {
          debug!("bounds {:?}", bounds);
          let center = bounds.center();
          let r = 8.0;
          let sphere =
            voxel::mosaic::solid::T {
              field: voxel::field::translation::T {
                translation: center.to_vec(),
                field: voxel::field::sphere::T {
                  radius: r,
                },
              },
              material: voxel::Material::Empty,
            };
          let r = sphere.field.field.radius + 1.0;
          let brush =
            voxel::brush::T {
              bounds:
                Aabb3::new(
                  {
                    let low = sphere.field.translation.add_v(&-Vector3::new(r, r, r));
                    Point3::new(low.x.floor() as i32, low.y.floor() as i32, low.z.floor() as i32)
                  },
                  {
                    let high = sphere.field.translation.add_v(&Vector3::new(r, r, r));
                    Point3::new(high.x.ceil() as i32, high.y.ceil() as i32, high.z.ceil() as i32)
                  },
                ),
              mosaic: Box::new(sphere) as Box<voxel::mosaic::T + Send>,
            };
          let brush: voxel::brush::T<Box<voxel::mosaic::T + Send>> = brush;
          update_gaia(ServerToGaia::Brush(brush));
        });
      },
    };
  })
}
