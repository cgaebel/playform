//! Camera structure and manipulation functions.

use gl;
use gl::types::*;
use cgmath;
use cgmath::{Matrix, Matrix3, Matrix4, Vector3, Point, Point3};
use std::f32::consts::PI;
use std::mem;
use yaglw::gl_context::GLContext;
use yaglw::shader::Shader;

/// Camera representation as 3 distinct matrices, as well as a position + two rotations.
pub struct Camera {
  #[allow(missing_docs)]
  pub position: Point3<f32>,
  #[allow(missing_docs)]
  pub lateral_rotation: f32,
  #[allow(missing_docs)]
  pub vertical_rotation: f32,

  // Projection matrix components

  #[allow(missing_docs)]
  pub translation: Matrix4<GLfloat>,
  #[allow(missing_docs)]
  pub rotation: Matrix4<GLfloat>,
  #[allow(missing_docs)]
  pub fov: Matrix4<GLfloat>,
}

impl Camera {
  /// This Camera sits at (0, 0, 0),
  /// maps [-1, 1] in x horizontally,
  /// maps [-1, 1] in y vertically,
  /// and [0, -1] in z in depth.
  pub fn unit() -> Camera {
    Camera {
      position: Point3::new(0.0, 0.0, 0.0),
      lateral_rotation: 0.0,
      vertical_rotation: 0.0,

      translation: Matrix4::identity(),
      rotation: Matrix4::identity(),
      fov: Matrix4::identity(),
    }
  }

  #[allow(missing_docs)]
  pub fn projection_matrix(&self) -> Matrix4<GLfloat> {
    self.fov.mul_m(&self.rotation).mul_m(&self.translation)
  }

  #[allow(missing_docs)]
  pub fn translate_to(&mut self, p: Point3<f32>) {
    self.position = p;
    self.translation = Matrix4::from_translation(&-p.to_vec());
  }

  /// Rotate about a given vector, by `r` radians.
  pub fn rotate(&mut self, v: &Vector3<f32>, r: f32) {
    let mat = Matrix3::from_axis_angle(v, -cgmath::rad(r));
    let mat =
      Matrix4::new(
        mat.x.x, mat.x.y, mat.x.z, 0.0,
        mat.y.x, mat.y.y, mat.y.z, 0.0,
        mat.z.x, mat.z.y, mat.z.z, 0.0,
        0.0,     0.0,     0.0,     1.0,
      );
    self.rotation.mul_self_m(&mat);
  }

  /// Rotate the camera around the y axis, by `r` radians. Positive is counterclockwise.
  pub fn rotate_lateral(&mut self, r: GLfloat) {
    self.lateral_rotation = self.lateral_rotation + r;
    self.rotate(&Vector3::new(0.0, 1.0, 0.0), r);
  }

  /// Changes the camera pitch by `r` radians. Positive is up.
  /// Angles that "flip around" (i.e. looking too far up or down)
  /// are sliently rejected.
  pub fn rotate_vertical(&mut self, r: GLfloat) {
    let new_rotation = self.vertical_rotation + r;

    if new_rotation < -PI / 2.0
    || new_rotation >  PI / 2.0 {
      return
    }

    self.vertical_rotation = new_rotation;

    let axis =
      Matrix3::from_axis_angle(
        &Vector3::new(0.0, 1.0, 0.0),
        cgmath::rad(self.lateral_rotation),
      );
    let axis = axis.mul_v(&Vector3::new(1.0, 0.0, 0.0));
    self.rotate(&axis, r);
  }
}

/// Set a shader's projection matrix to match that of a camera.
pub fn set_camera(shader: &mut Shader, gl: &mut GLContext, c: &Camera) {
  let projection_matrix = shader.get_uniform_location("projection_matrix");
  shader.use_shader(gl);
  unsafe {
    let val = c.projection_matrix();
    let ptr = mem::transmute(&val);
    gl::UniformMatrix4fv(projection_matrix, 1, 0, ptr);
  }
}
