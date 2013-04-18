/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: gl/ttf/renderer.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      A TTF font renderer.
*/

use gl::shader::Shader;
use gl::camera::Camera;
use gl = opengles::gl2;
use font::Font;
use math::{ Vec2f, Mat4x4 };

#[path = "../util.rs"]
mod util;
#[macro_escape]
#[path = "../check_internal.rs"]
mod check_internal;

struct Renderer
{
  vbo: gl::GLuint,
  shader: Shader,
  proj_loc: gl::GLint,
}

impl Renderer
{
  pub fn new() -> Renderer
  {
    let vert_shader =
      "#version 330 core

      uniform mat4 proj; 
      uniform mat4 world; 

      layout (location = 0) in vec4 in_coord; 

      out vec2 trans_coord; 

      void main() 
      { 
        gl_Position = proj * vec4(in_coord.xy, -10.0, 1); 
        trans_coord = in_coord.zw; 
      };";

    let frag_shader =
      "#version 330 core

      uniform sampler2D tex0;
      uniform vec4 color0;

      in vec2 trans_coord;
      out vec4 out_color;

      void main()
      {
        //out_color = vec4(1, 1, 1, texture2D(tex0, trans_coord).r) * color0;
        out_color = vec4(1, 1, 1, texture2D(tex0, trans_coord).r);
      }";

    let mut renderer = Renderer
    {
        vbo: 0,
        shader: Shader::new(vert_shader, frag_shader),
        proj_loc: 0,
    };
    renderer.proj_loc = renderer.shader.get_uniform_location(~"proj");
    let tex_loc = renderer.shader.get_uniform_location("tex0"); 
    renderer.shader.bind();
    renderer.shader.update_uniform_i32(tex_loc, 0);

    renderer.vbo = check!(gl::gen_buffers(1))[0];

    renderer
  }

  pub fn begin(&mut self, camera: &Camera)
  {
    check!(gl::disable(gl::DEPTH_TEST));

    check!(gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
    check!(gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));

    /* Enable transparency. */
    check!(gl::enable(gl::BLEND));
    check!(gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

    self.shader.bind();
    let proj =  Mat4x4::new_orthographic(0.0, camera.window_size.x as f32, camera.window_size.y as f32, 0.0,  1.0, 100.0);
    self.shader.update_uniform_mat(self.proj_loc, &proj);
  }

  pub fn end(&mut self)
  {
    check!(gl::enable(gl::DEPTH_TEST));
    check!(gl::disable(gl::BLEND));
  }

  pub fn render(&mut self, text: &str, pos: Vec2f, font: &Font)
  {
    check!(gl::active_texture(gl::TEXTURE0));
    check!(gl::bind_texture(gl::TEXTURE_2D, font.texture_atlas));

    check!(gl::bind_buffer(gl::ARRAY_BUFFER, self.vbo));
    check!(gl::vertex_attrib_pointer_f32(0, 4, false, 0, 0));
    check!(gl::enable_vertex_attrib_array(0));

    struct Point
    {
      x: f32, y: f32,
      u: f32, v: f32
    }
    impl Point
    {
      pub fn new(nx: f32, ny: f32, nu: f32, nv: f32) -> Point
      { Point { x: nx, y: ny, u: nu, v: nv } }
    }
    let mut coords = vec::with_capacity::<Point>(text.len());

    let mut temp_pos = pos;
    temp_pos.y += font.height as f32;

    let mut count = 0;
    for text.each |curr|
    {
      let glyph = match font.glyphs.find(&curr)
      {
        Some(g) => g,
        None => fail!(fmt!("Invalid char (%?) in font %? len %?", curr, font.file, font.glyphs.len()))
      };

      let end_x = temp_pos.x + glyph.offset.x;
      let end_y = -temp_pos.y - (glyph.dimensions.y - glyph.offset.y);
      let end_w = glyph.dimensions.x; /* TODO: Use this everywhere. */
      let end_h = glyph.dimensions.y;

      temp_pos.x += glyph.advance.x; 
      temp_pos.y += glyph.advance.y; 

      /* Skip empty glyphs. */
      if end_w <= 0.1 || end_h <= 0.1
      { loop; }

      coords.push(Point::new(end_x, -end_y - end_h, glyph.tex.x, glyph.tex.y));
      coords.push(Point::new(end_x, -end_y, glyph.tex.x, glyph.tex.y + (glyph.dimensions.y / (font.atlas_dimensions.y as f32))));
      coords.push(Point::new(end_x + end_w, -end_y, glyph.tex.x + (glyph.dimensions.x / (font.atlas_dimensions.x as f32)), glyph.tex.y + (glyph.dimensions.y / (font.atlas_dimensions.y as f32))));
      coords.push(Point::new(end_x, -end_y - end_h, glyph.tex.x, glyph.tex.y));
      coords.push(Point::new(end_x + end_w, -end_y, glyph.tex.x + (glyph.dimensions.x / (font.atlas_dimensions.x as f32)), glyph.tex.y + (glyph.dimensions.y / (font.atlas_dimensions.y as f32))));
      coords.push(Point::new(end_x + end_w, -end_y - end_h, glyph.tex.x + (glyph.dimensions.x / (font.atlas_dimensions.x as f32)), glyph.tex.y));
      count += 6;
    }

    check!(gl::buffer_data(gl::ARRAY_BUFFER, coords, gl::DYNAMIC_DRAW)); /* TODO: STREAM */
    check!(gl::draw_arrays(gl::TRIANGLES, 0, count));
  }
}
