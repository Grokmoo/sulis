//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

pub mod event;
pub use self::event::Event;

mod glium_adapter;

mod input_action;
pub use self::input_action::InputAction;

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

use std::io::Error;
use std::rc::Rc;
use std::cell::{RefCell, Ref};

use extern_image::{ImageBuffer, Rgba};

use config::{CONFIG, IOAdapter};
use ui::{Widget, Color};
use resource::Sprite;

pub trait MainLoopUpdater {
    fn update(&self);

    fn is_exit(&self) -> bool;
}

pub trait IO {
    fn process_input(&mut self, root: Rc<RefCell<Widget>>);

    fn render_output(&mut self, root: Ref<Widget>, millis: u32);
}

pub trait GraphicsRenderer {
    fn draw(&mut self, draw_list: DrawList);

    fn draw_to_texture(&mut self, texture_id: &str, draw_list: DrawList);

    fn register_texture(&mut self, id: &str, image: ImageBuffer<Rgba<u8>, Vec<u8>>,
                        min_filter: TextureMinFilter, mag_filter: TextureMagFilter);

    fn clear_texture(&mut self, id: &str);

    fn has_texture(&self, id: &str) -> bool;
}

#[derive(Debug, Copy, Clone)]
pub enum TextureMagFilter {
    Nearest,
    Linear,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureMinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Debug, Copy, Clone)]
pub enum DrawListKind {
    Font,
    Sprite,
}

#[derive(Debug, Clone)]
pub struct DrawList {
    pub quads: Vec<[Vertex; 4]>,
    pub texture: String,
    pub texture_mag_filter: TextureMagFilter,
    pub texture_min_filter: TextureMinFilter,
    pub kind: DrawListKind,
    pub color_filter: [f32; 4],
    pub scale: [f32; 2],
}

pub const GFX_BORDER_SCALE: f32 = 0.75;

impl Default for DrawList {
    fn default() -> DrawList {
        DrawList {
            quads: Vec::new(),
            texture: String::new(),
            texture_mag_filter: TextureMagFilter::Linear,
            texture_min_filter: TextureMinFilter::Linear,
            kind: DrawListKind::Sprite,
            color_filter: [1.0, 1.0, 1.0, 1.0],
            scale: [1.0, 1.0],
        }
    }
}

impl DrawList {
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    /// Creates an empty DrawList.  Attempting to draw an empty DrawList will most
    /// likely result in a panic, you must use `append` to add vertices to this list
    /// if you intend to draw it.
    pub fn empty_sprite() -> DrawList {
        Default::default()
    }

    pub fn from_font(texture: &str, quads: Vec<[Vertex; 4]>) -> DrawList {
        DrawList {
            texture: texture.to_string(),
            quads,
            kind: DrawListKind::Font,
            ..Default::default()
        }
    }

    pub fn from_texture_id(id: &str, tex_coords: &[f32; 8], x: f32,
                           y: f32, w: f32, h: f32) -> DrawList {
        let x_min = x;
        let y_max = CONFIG.display.height as f32 - y;
        let x_max = x_min + w;
        let y_min = y_max - h;
        let tc = tex_coords;

        let quads = vec![[
            Vertex { position: [ x_min, y_max ], tex_coords: [tc[0], tc[1]] },
            Vertex { position: [ x_min, y_min ], tex_coords: [tc[2], tc[3]] },
            Vertex { position: [ x_max, y_max ], tex_coords: [tc[4], tc[5]] },
            Vertex { position: [ x_max, y_min ], tex_coords: [tc[6], tc[7]] },
        ]];

        DrawList {
            texture: id.to_string(),
            quads,
            ..Default::default()
        }
    }

    pub fn from_sprite_f32(sprite: &Rc<Sprite>, x: f32, y: f32, w: f32, h: f32) -> DrawList {
        DrawList::from_texture_id(&sprite.id, &sprite.tex_coords, x, y, w, h)
    }

    pub fn from_sprite(sprite: &Rc<Sprite>, x: i32, y: i32, w: i32, h: i32) -> DrawList {
        DrawList::from_sprite_f32(sprite, x as f32, y as f32, w as f32, h as f32)
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale = [scale_x, scale_y];
    }

    pub fn set_color(&mut self, color: Color) {
        self.color_filter = [color.r, color.g, color.b, color.a];
    }

    /// appends the contents of the other drawlist to this one, moving
    /// the vertex data out into this DrawList's vertex data.
    pub fn append(&mut self, other: &mut DrawList) {
        self.quads.append(&mut other.quads);
        if self.texture.is_empty() {
            self.texture = other.texture.to_string();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub fn create() -> Result<Box<IO>, Error> {
    match CONFIG.display.adapter {
        IOAdapter::Auto => get_auto_adapter(),
        IOAdapter::Glium => get_glium_adapter(),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_auto_adapter() -> Result<Box<IO>, Error> {
    get_glium_adapter()
}

pub fn get_glium_adapter() -> Result<Box<IO>, Error> {
    Ok(Box::new(glium_adapter::GliumDisplay::new()))
}

#[cfg(target_os = "windows")]
pub fn get_auto_adapter() -> Result<Box<IO>, Error> {
    get_glium_adapter()
}
