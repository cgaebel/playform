use cgmath::{Point3, Vector3, EuclideanVector};
use noise::{Seed, Brownian2, Brownian3, perlin2, perlin3};

use voxel;

pub struct T {
  pub height: Brownian2<f64, fn (&Seed, &[f64; 2]) -> f64>,
  pub features: Brownian3<f64, fn (&Seed, &[f64; 3]) -> f64>,
  pub seed: Seed,
}

impl T {
  pub fn new(seed: Seed) -> T {
    let perlin2: fn(&Seed, &[f64; 2]) -> f64 = perlin2;
    let perlin3: fn(&Seed, &[f64; 3]) -> f64 = perlin3;
    T {
      seed: seed,
      height:
        Brownian2::new(perlin2, 5)
        .frequency(1.0 / 4.0)
        .persistence(2.0)
        .lacunarity(1.0 / 2.0)
      ,
      features:
        Brownian3::new(perlin3, 2)
        .frequency(1.0 / 32.0)
        .persistence(8.0)
        .lacunarity(1.0 / 4.0)
      ,
    }
  }
}

fn signed_density(this: &T, p: &Point3<f32>) -> f32 {
  let height = this.height.apply(&this.seed, &[p.x as f64, p.z as f64]);
  let height = height as f32;
  let heightmap_density = height - p.y;

  let feature_density = this.features.apply(&this.seed, &[p.x as f64, p.y as f64, p.z as f64]) * 8.0;
  let feature_density = feature_density as f32;

  heightmap_density + feature_density
}

impl voxel::field::T for T {
  fn density(this: &T, p: &Point3<f32>) -> f32 {
    signed_density(this, p).abs()
  }

  fn normal(this: &Self, p: &Point3<f32>) -> Vector3<f32> {
    // Use density differential in each dimension as an approximation of the normal.

    let delta = 0.01;

    macro_rules! differential(($d:ident) => {{
      let high: f32 = {
        let mut p = *p;
        p.$d += delta;
        signed_density(this, &p)
      };
      let low: f32 = {
        let mut p = *p;
        p.$d -= delta;
        signed_density(this, &p)
      };
      high - low
    }});

    let v = Vector3::new(differential!(x), differential!(y), differential!(z));
    // Negate because we're leaving the volume when density is decreasing.
    let v = -v;
    v.normalize()
  }

  fn material(this: &Self, p: &Point3<f32>) -> Option<voxel::Material> {
    Some(
      if signed_density(this, p) >= 0.0 {
        voxel::Material::Terrain
      } else {
        voxel::Material::Empty
      }
    )
  }
}
