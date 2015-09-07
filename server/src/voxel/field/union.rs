use cgmath::{Point3, Vector3};
use std::f32;

use voxel::field;

pub struct T {
  pub components: Vec<Box<field::Dispatch>>,
}

unsafe impl Send for T {}

pub fn new() -> T {
  T {
    components: Vec::new(),
  }
}

pub fn push<Field>(this: &mut T, field: Field) 
  where Field: field::T + 'static,
{
  this.components.push(Box::new(field));
}

impl field::T for T {
  fn density(this: &Self, p: &Point3<f32>) -> f32 {
    assert!(this.components.len() > 0);
    this.components.iter().fold(
      f32::NEG_INFINITY, 
      |max, shape| f32::max(max, shape.density(p)),
    )
  }

  fn normal(this: &Self, p: &Point3<f32>) -> Vector3<f32> {
    assert!(this.components.len() > 0);
    let (_, normal) =
      this.components.iter().fold(
        (f32::NEG_INFINITY, Vector3::new(0.0, 0.0, 0.0)), 
        |(max, normal), shape| {
          let d = shape.density(p);
          if d > max {
            (d, shape.normal(p))
          } else {
            (max, normal)
          }
        },
      );
    normal
  }

  fn material(this: &Self, p: &Point3<f32>) -> Option<::voxel::Material> {
    assert!(this.components.len() > 0);
    for shape in this.components.iter() {
      match shape.material(p) {
        None => {},
        Some(material) => return Some(material),
      }
    }
    None
  }
}
