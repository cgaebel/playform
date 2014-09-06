#version 330 core

uniform mat4 projection_matrix;

layout(points) in;
layout(triangle_strip, max_vertices = 36) out;

out vec2 tex_position;
out vec3 world_position;
out vec3 normal;

// Remember: x increases to the right, y increases up, and z becomes more
// negative as depth from the viewer increases.

void main() {
  float d = 0.5;
  float x1 = gl_in[0].gl_Position.x;
  float y1 = gl_in[0].gl_Position.y;
  float z1 = gl_in[0].gl_Position.z;
  float x2 = x1 + d;
  float y2 = y1 + d;
  float z2 = z1 + d;

  // hacky little solution so that we don't index right onto the edge of a
  // texture; if we do, we get edges showing up in rendering.
  d = 0.01;

  // front
  gl_Position = projection_matrix * vec4(x1, y1, z2, 1);
  world_position = vec3(x1, y1, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.00 + d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.25 - d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z2, 1);
  world_position = vec3(x1, y2, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.25 - d, 0.50 - d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x1, y1, z2, 1);
  world_position = vec3(x1, y1, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.00 + d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z2, 1);
  world_position = vec3(x2, y1, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.00 + d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(0.0, 0.0, 1.0);
  tex_position = vec2(0.25 - d, 0.25 + d);
  EmitVertex();
  EndPrimitive();
  // left
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.00 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z2, 1);
  world_position = vec3(x1, y2, z2);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.25 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z1, 1);
  world_position = vec3(x1, y2, z1);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.00 + d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.00 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y1, z2, 1);
  world_position = vec3(x1, y1, z2);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.25 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z2, 1);
  world_position = vec3(x1, y2, z2);
  normal = vec3(-1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.25 - d);
  EmitVertex();
  EndPrimitive();
  // top
  gl_Position = projection_matrix * vec4(x1, y2, z1, 1);
  world_position = vec3(x1, y2, z1);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.25 + d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.50 - d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z1, 1);
  world_position = vec3(x2, y2, z1);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.25 + d, 0.50 - d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x1, y2, z1, 1);
  world_position = vec3(x1, y2, z1);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.25 + d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z2, 1);
  world_position = vec3(x1, y2, z2);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.50 - d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(0.0, 1.0, 0.0);
  tex_position = vec2(0.50 - d, 0.50 - d);
  EmitVertex();
  EndPrimitive();
  // back
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.75 - d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z1, 1);
  world_position = vec3(x2, y2, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.50 + d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z1, 1);
  world_position = vec3(x2, y1, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.75 - d, 0.25 + d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.75 - d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y2, z1, 1);
  world_position = vec3(x1, y2, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.50 + d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z1, 1);
  world_position = vec3(x2, y2, z1);
  normal = vec3(0.0, 0.0, -1.0);
  tex_position = vec2(0.50 + d, 0.25 + d);
  EmitVertex();
  EndPrimitive();
  // right
  gl_Position = projection_matrix * vec4(x2, y1, z1, 1);
  world_position = vec3(x2, y1, z1);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.75 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.50 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z2, 1);
  world_position = vec3(x2, y1, z2);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.50 + d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x2, y1, z1, 1);
  world_position = vec3(x2, y1, z1);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.75 - d, 0.75 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z1, 1);
  world_position = vec3(x2, y2, z1);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.75 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y2, z2, 1);
  world_position = vec3(x2, y2, z2);
  normal = vec3(1.0, 0.0, 0.0);
  tex_position = vec2(0.50 + d, 0.50 + d);
  EmitVertex();
  EndPrimitive();
  // bottom
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(0.75 + d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z2, 1);
  world_position = vec3(x2, y1, z2);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(1.00 - d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x1, y1, z2, 1);
  world_position = vec3(x1, y1, z2);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(1.00 - d, 0.50 - d);
  EmitVertex();
  EndPrimitive();
  gl_Position = projection_matrix * vec4(x1, y1, z1, 1);
  world_position = vec3(x1, y1, z1);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(0.75 + d, 0.50 - d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z1, 1);
  world_position = vec3(x2, y1, z1);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(0.75 + d, 0.25 + d);
  EmitVertex();
  gl_Position = projection_matrix * vec4(x2, y1, z2, 1);
  world_position = vec3(x2, y1, z2);
  normal = vec3(0.0, -1.0, 0.0);
  tex_position = vec2(1.00 - d, 0.25 + d);
  EmitVertex();
  EndPrimitive();
}
