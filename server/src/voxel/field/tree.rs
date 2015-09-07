/// A tree is comprised of a cylindrical trunk, a spherical bunch of leaves, and a spherical
/// rounding to the bottom of the trunk.

use cgmath::{Point, Point3, Vector3};

use voxel;
use voxel::field;

mod pillar {
  use cgmath::{Point, Point3, Vector3, EuclideanVector};

  use voxel;
  use voxel::field;

  pub struct T {
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub material: voxel::Material,
  }

  fn signed_density(this: &T, p: &Point3<f32>) -> f32 {
    let d = Point3::new(this.x, p.y, this.z).sub_p(p);
    this.radius*this.radius - d.length2()
  }

  impl field::T for T {
    fn density(this: &T, p: &Point3<f32>) -> f32 {
      signed_density(this, p).abs()
    }

    fn normal(this: &T, p: &Point3<f32>) -> Vector3<f32> {
      Point3::new(this.x, p.y, this.z).sub_p(p).normalize()
    }

    fn material(this: &T, p: &Point3<f32>) -> Option<voxel::Material> {
      if signed_density(this, p) >= 0.0 {
        Some(this.material)
      } else {
        None
      }
    }
  }
}

pub struct T {
  union: field::union::T,
}

unsafe impl Send for T {}

pub fn new(
  // Bottom-center of the trunk
  bottom: Point3<f32>, 
  trunk_height: f32, 
  trunk_radius: f32, 
  leaf_radius: f32,
) -> T {
  let leaf_center = bottom.add_v(&Vector3::new(0.0, trunk_height, 0.0));
  let trunk_center = bottom.add_v(&Vector3::new(0.0, trunk_height / 2.0, 0.0));

  let leaves =
    field::sphere::T {
      center: leaf_center,
      radius: leaf_radius,
      material: voxel::Material::Leaves,
    };

  let mut trunk = field::intersection::new();
  field::intersection::push(
    &mut trunk,
    pillar::T {
      x: bottom.x,
      z: bottom.z,
      radius: trunk_radius,
      material: voxel::Material::Bark,
    },
  );
  field::intersection::push(
    &mut trunk,
    field::sphere::T {
      center: trunk_center,
      radius: trunk_height / 2.0,
      material: voxel::Material::Bark,
    },
  );

  let mut union = field::union::new();
  field::union::push(&mut union, leaves);
  field::union::push(&mut union, trunk);

  T {
    union: union,
  }
}

impl field::T for T {
  fn density(this: &Self, p: &Point3<f32>) -> f32 {
    field::T::density(&this.union, p)
  }

  fn normal(this: &Self, p: &Point3<f32>) -> Vector3<f32> {
    field::T::normal(&this.union, p)
  }

  fn material(this: &Self, p: &Point3<f32>) -> Option<::voxel::Material> {
    field::T::material(&this.union, p)
  }
}
