//! Defines the messages passed between client and server.

use cgmath::{Aabb3, Vector2, Vector3, Point3};
use std::default::Default;
use std::ops::Add;

use block_position::BlockPosition;
use entity::EntityId;
use lod::LODIndex;
use terrain_block::TerrainBlock;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, RustcEncodable, RustcDecodable)]
/// Unique client ID.
pub struct ClientId(u32);

impl Default for ClientId {
  fn default() -> ClientId {
    ClientId(0)
  }
}

impl Add<u32> for ClientId {
  type Output = ClientId;

  fn add(self, rhs: u32) -> ClientId {
    let ClientId(i) = self;
    ClientId(i + rhs)
  }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
/// TerrainBlock plus identifying info, e.g. for transmission between server and client.
pub struct TerrainBlockSend {
  #[allow(missing_docs)]
  pub position: BlockPosition,
  #[allow(missing_docs)]
  pub block: TerrainBlock,
  #[allow(missing_docs)]
  pub lod: LODIndex,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
/// Messages the client sends to the server.
pub enum ClientToServer {
  /// Notify the server that the client exists, and provide a "return address".
  Init(String),
  /// Ping
  Ping(ClientId),
  /// Ask the server to create a new player.
  AddPlayer(ClientId),
  /// Add a vector the player's acceleration.
  Walk(EntityId, Vector3<f32>),
  /// Rotate the player by some amount.
  RotatePlayer(EntityId, Vector2<f32>),
  /// [Try to] start a jump for the player.
  StartJump(EntityId),
  /// [Try to] stop a jump for the player.
  StopJump(EntityId),
  /// Ask the server to send a block of terrain.
  RequestBlock(ClientId, BlockPosition, LODIndex),
  /// Brush-remove where the player's looking.
  Add(EntityId),
  /// Brush-add at where the player's looking.
  Remove(EntityId),
}

/// Why a block is being sent to a client.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum BlockReason {
  /// The client asked for it.
  Requested,
  /// The block has been updated.
  Updated,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
/// Messages the server sends to the client.
pub enum ServerToClient {
  /// Provide the client a unique id to tag its messages.
  LeaseId(ClientId),
  /// Ping
  Ping,

  /// Complete an AddPlayer request.
  PlayerAdded(EntityId, Point3<f32>),
  /// Update a player's position.
  UpdatePlayer(EntityId, Aabb3<f32>),

  /// Update the client's view of a mob with a given mesh.
  UpdateMob(EntityId, Aabb3<f32>),

  /// The sun as a [0, 1) portion of its cycle.
  UpdateSun(f32),

  /// Provide a block of terrain to a client.
  Block(TerrainBlockSend, BlockReason),
}
