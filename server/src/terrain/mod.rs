mod generate;
mod heightmap;

pub use noise::Seed;

use cgmath::Aabb;
use std::collections::hash_map::HashMap;
use std::iter::range_inclusive;
use std::sync::Mutex;
use stopwatch::TimerSet;

use common::block_position::BlockPosition;
use common::entity::EntityId;
use common::id_allocator::IdAllocator;
use common::lod::LODIndex;
use common::terrain_block;
use common::terrain_block::TerrainBlock;

pub mod voxel {
  pub use ::voxel::impls::surface_vertex::*;

  pub mod tree {
    pub use ::voxel::tree::TreeBody::*;
    pub type T = ::voxel::tree::T<super::T>;
    pub type TreeBody = ::voxel::tree::TreeBody<super::T>;
    pub type Branches = ::voxel::tree::Branches<super::T>;
  }
}

pub struct MipMesh {
  pub lods: Vec<Option<TerrainBlock>>,
}

impl MipMesh {
  pub fn get_mut<'a>(&'a mut self, i: usize) -> &'a mut Option<TerrainBlock> {
    for _ in range_inclusive(self.lods.len(), i) {
      self.lods.push(None);
    }
    self.lods.get_mut(i).unwrap()
  }
}

pub struct MipMeshMap(pub HashMap<BlockPosition, MipMesh>);

impl MipMeshMap {
  pub fn new() -> MipMeshMap {
    MipMeshMap(HashMap::new())
  }

  pub fn get<'a>(&'a mut self, position: &BlockPosition) -> Option<&'a MipMesh> {
    self.0.get(position)
  }

  pub fn get_mut<'a>(&'a mut self, position: &BlockPosition) -> &'a mut MipMesh {
    self.0
      .entry(*position)
      .or_insert_with(|| {
        MipMesh {
          lods: Vec::new(),
        }
      })
  }
}

/// This struct contains and lazily generates the world's terrain.
pub struct Terrain {
  pub heightmap: heightmap::T,
  // all the blocks that have ever been created.
  pub all_blocks: MipMeshMap,
  pub voxels: voxel::tree::T,
}

impl Terrain {
  pub fn new(terrain_seed: Seed) -> Terrain {
    Terrain {
      heightmap: heightmap::T::new(terrain_seed),
      all_blocks: MipMeshMap::new(),
      voxels: voxel::tree::T::new(),
    }
  }

  // TODO: Allow this to be performed in such a way that self is only briefly locked.
  pub fn load<'a>(
    &'a mut self,
    timers: &TimerSet,
    id_allocator: &Mutex<IdAllocator<EntityId>>,
    position: &BlockPosition,
    lod_index: LODIndex,
  ) -> &'a TerrainBlock
  {
    let mip_mesh = self.all_blocks.get_mut(position);
    let mesh = mip_mesh.get_mut(lod_index.0 as usize);
    if mesh.is_none() {
      *mesh = Some(
        generate::generate_block(
          timers,
          id_allocator,
          &self.heightmap,
          &mut self.voxels,
          position,
          lod_index,
        )
      );
    }
    mesh.as_ref().unwrap()
  }

  pub fn remove<F, Brush>(
    &mut self,
    timers: &TimerSet,
    id_allocator: &Mutex<IdAllocator<EntityId>>,
    brush: &Brush,
    brush_bounds: &::voxel::brush::Bounds,
    mut block_changed: F,
  ) where
    F: FnMut(&TerrainBlock, &BlockPosition, LODIndex),
    Brush: voxel::brush::T,
  {
    self.voxels.remove(brush, brush_bounds);

    macro_rules! block_range(($d:ident) => {{
      let low = brush_bounds.min().$d >> terrain_block::LG_WIDTH;
      let high = brush_bounds.max().$d >> terrain_block::LG_WIDTH;
      range_inclusive(low, high)
    }});

    for x in block_range!(x) {
    for y in block_range!(y) {
    for z in block_range!(z) {
      let position = BlockPosition::new(x, y, z);
      let mip_mesh = self.all_blocks.get_mut(&position);

      for (i, mesh) in mip_mesh.lods.iter_mut().enumerate() {
        match mesh {
          &mut None => {},
          &mut Some(ref mut mesh) => {
            let lod_index = LODIndex(i as u32);
            *mesh =
              generate::generate_block(
                timers,
                id_allocator,
                &self.heightmap,
                &mut self.voxels,
                &position,
                lod_index,
              )
            ;

            block_changed(mesh, &position, lod_index);
          },
        }
      }
    }}}
  }
}