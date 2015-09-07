use cgmath::{Point3, Aabb3, Vector3};

use voxel;

pub mod union;
pub mod intersection;

pub mod sphere;
pub mod tree;

pub type Bounds = Aabb3<i32>;

/// A density field that also defines materials. This does not need to be defined everywhere.
pub trait T {
  /// The material density at a given point. This should be nonnegative!
  fn density(this: &Self, p: &Point3<f32>) -> f32;

  /// The surface normal at a given point.
  fn normal(this: &Self, p: &Point3<f32>) -> Vector3<f32>;

  /// The material at this point.
  fn material(this: &Self, p: &Point3<f32>) -> Option<voxel::Material>;
}

#[allow(missing_docs)]
/// Dispatch to voxel::field::T.
pub trait Dispatch {
  fn density(&self, p: &Point3<f32>) -> f32;
  fn normal(&self, p: &Point3<f32>) -> Vector3<f32>;
  fn material(&self, p: &Point3<f32>) -> Option<voxel::Material>;
}

impl<X> Dispatch for X where X: T {
  fn density(&self, p: &Point3<f32>) -> f32 {
    T::density(self, p)
  }

  fn normal(&self, p: &Point3<f32>) -> Vector3<f32> {
    T::normal(self, p)
  }

  fn material(&self, p: &Point3<f32>) -> Option<voxel::Material> {
    T::material(self, p)
  }
}

