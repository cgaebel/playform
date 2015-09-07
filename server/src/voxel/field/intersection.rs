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
      f32::INFINITY, 
      |min, shape| f32::min(min, shape.density(p)),
    )
  }

  fn normal(this: &Self, p: &Point3<f32>) -> Vector3<f32> {
    assert!(this.components.len() > 0);
    let (_, normal) =
      this.components.iter().fold(
        (f32::INFINITY, Vector3::new(0.0, 0.0, 0.0)), 
        |(min, normal), shape| {
          let d = shape.density(p);
          if d > min {
            (d, shape.normal(p))
          } else {
            (min, normal)
          }
        },
      );
    normal
  }


  fn material(this: &Self, p: &Point3<f32>) -> Option<::voxel::Material> {
    assert!(this.components.len() > 0);
    let mut material = None;
    for shape in this.components.iter() {
      match shape.material(p) {
        None => return None,
        Some(m) => material = Some(m),
      }
    }
    material
  }
}
