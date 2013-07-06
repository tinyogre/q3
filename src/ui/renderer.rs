/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: ui/renderer.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      A UI component renderer.
*/

use gl;
use math;
use TTF_Renderer = super::ttf::Renderer;
use TTF_Font = super::ttf::Font;
use gl2 = opengles::gl2;

#[path = "../gl/check.rs"]
mod check;

struct Renderer
{
  vao: gl2::GLuint,
  vbo: gl2::GLuint,

  /* Shader uniforms. */
  shader: @gl::Shader,
  world: math::Mat4x4,
  tex_world: math::Mat4x4,

  /* Shader uniform locations. */
  proj_loc: gl2::GLint,
  world_loc: gl2::GLint,
  alpha_loc: gl2::GLint,
  tex_world_loc: gl2::GLint,
  texture0_loc: gl2::GLint,

  /* Font support. */
  font_renderer: TTF_Renderer,

  /* Window. */
  window_size: math::Vec2i,
}

impl Renderer
{
  pub fn new() -> Renderer
  {
    let mut renderer = Renderer
    {
      vao: 0,
      vbo: 0,

      shader: gl::Shader_Builder::new_with_files("data/shaders/ui.vert", "data/shaders/ui.frag"),
      world: math::Mat4x4::new(),
      tex_world: math::Mat4x4::new(),

      proj_loc: 0,
      world_loc: 0,
      alpha_loc: 0,
      tex_world_loc: 0,
      texture0_loc: 0,

      font_renderer: TTF_Renderer::new(),

      window_size: math::Vec2i::zero(),
    };

    renderer.proj_loc = renderer.shader.get_uniform_location("proj");
    renderer.world_loc = renderer.shader.get_uniform_location("world");
    renderer.alpha_loc = renderer.shader.get_uniform_location("alpha");
    renderer.tex_world_loc = renderer.shader.get_uniform_location("tex_world");
    renderer.texture0_loc = renderer.shader.get_uniform_location("texture0"); 
    renderer.shader.bind();
    renderer.shader.update_uniform_i32(renderer.texture0_loc, 0);

    /* VAO */
    let name = check!(gl2::gen_vertex_arrays(1));
    assert!(name.len() == 1);
    renderer.vao = name[0];
    check!(gl2::bind_vertex_array(renderer.vao));

    /* VBO */
    let name = check!(gl2::gen_buffers(1));
    assert!(name.len() == 1);
    renderer.vbo = name[0];
    check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, renderer.vbo));

    struct Point
    {
      x: f32, y: f32,
      u: f32, v: f32
    }
    impl Point
    {
      #[inline(always)]
      pub fn new(nx: f32, ny: f32, nu: f32, nv: f32) -> Point
      { Point { x: nx, y: ny, u: nu, v: nv } }
    }
    let data =
    [
      /*(X , Y) (U , V)*/
      Point::new(0.0, 0.0, 0.0, 0.0),
      Point::new(0.0, 1.0, 0.0, 1.0),
      Point::new(1.0, 1.0, 1.0, 1.0),
      Point::new(1.0, 0.0, 1.0, 0.0),
    ];
    check!(gl2::buffer_data(gl2::ARRAY_BUFFER, data, gl2::STATIC_DRAW));
    check!(gl2::enable_vertex_attrib_array(0));

    renderer
  }

  #[inline(always)]
  pub fn begin(&mut self, camera: &gl::Camera)
  {
    check!(gl2::disable(gl2::DEPTH_TEST));

    /* Enable transparency. */
    check!(gl2::enable(gl2::BLEND));

    /* Update the projection information. */
    self.window_size = camera.window_size;
    let proj = math::Mat4x4::new_orthographic(0.0, self.window_size.x as f32, self.window_size.y as f32, 0.0,  1.0, 100.0);

    self.font_renderer.shader.bind();
    self.font_renderer.shader.update_uniform_mat(self.font_renderer.proj_loc, &proj);
    
    self.shader.bind();
    self.shader.update_uniform_mat(self.proj_loc, &proj);
  }

  #[inline(always)]
  pub fn end(&mut self)
  {
    check!(gl2::enable(gl2::DEPTH_TEST));
    check!(gl2::disable(gl2::BLEND));
  }

  pub fn render_texture(&mut self, tex: &gl::Texture, pos: &math::Vec2f)
  {
    self.world = math::Mat4x4::new_scale(tex.size.x as f32, tex.size.y as f32, 1.0);
    self.world = self.world * math::Mat4x4::new_translation(pos.x, pos.y, 0.0);
    self.shader.update_uniform_mat(self.world_loc, &self.world);

    self.tex_world.identity();
    self.shader.update_uniform_mat(self.tex_world_loc, &self.tex_world);

    self.render(tex);
  }

  pub fn render_texture_scale_clamp(&mut self, tex: &gl::Texture, pos: &math::Vec2f, scale: &math::Vec2f)
  {
    self.world = math::Mat4x4::new_scale(scale.x, scale.y, 1.0);
    self.world = self.world * math::Mat4x4::new_translation(pos.x, pos.y, 0.0);
    self.shader.update_uniform_mat(self.world_loc, &self.world);

    self.tex_world.identity();
    self.shader.update_uniform_mat(self.tex_world_loc, &self.tex_world);

    self.render(tex);
  }

  pub fn render_font(&mut self, text: &str, pos: math::Vec2f, font: &TTF_Font)
  {
    self.font_renderer.shader.bind();
    self.font_renderer.render(text, pos, font);
    self.shader.bind();
  }

  priv fn render(&mut self, tex: &gl::Texture)
  {
    tex.bind(0);

    check!(gl2::bind_vertex_array(self.vao));
    check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, self.vbo));
    check!(gl2::enable_vertex_attrib_array(0));
    check!(gl2::vertex_attrib_pointer_f32(0, 4, false, 0, 0));

    check!(gl2::draw_arrays(gl2::TRIANGLE_FAN, 0, 4));

    tex.unbind();

    check!(gl2::disable_vertex_attrib_array(0));
    check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));
    check!(gl2::bind_vertex_array(0));
  }
}

