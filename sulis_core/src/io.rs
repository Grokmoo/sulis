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

pub mod audio;
pub use self::audio::{create_audio_device, Audio, AudioDevice, AudioDeviceInfo, SoundSource};

pub mod event;
pub use self::event::Event;

mod glium_adapter;

mod input_action;
pub use self::input_action::{InputAction, InputActionKind, InputActionState};

pub mod keyboard_event;
pub use self::keyboard_event::KeyboardEvent;

use std::cell::RefCell;
use std::io::Error;
use std::rc::Rc;

use crate::extern_image::{ImageBuffer, Rgba};

use crate::config::Config;
use crate::resource::Sprite;
use crate::ui::{Color, Widget};
use crate::util::{Point, Rect, Scale, Size};

#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    pub name: String,
    pub index: usize,
    pub resolutions: Vec<Resolution>,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub monitor_size: bool,
}

pub trait ControlFlowUpdater {
    fn update(&mut self, millis: u32) -> Rc<RefCell<Widget>>;

    fn root(&self) -> Rc<RefCell<Widget>>;

    fn recreate_window(&mut self) -> bool;

    fn is_exit(&self) -> bool;
}

pub trait GraphicsRenderer {
    fn draw(&mut self, draw_list: DrawList);

    fn draw_to_texture(&mut self, texture_id: &str, draw_list: DrawList);

    fn register_texture(
        &mut self,
        id: &str,
        image: ImageBuffer<Rgba<u8>, Vec<u8>>,
        min_filter: TextureMinFilter,
        mag_filter: TextureMagFilter,
    );

    fn clear_texture(&mut self, id: &str);

    fn clear_texture_region(&mut self, id: &str, min_x: i32, min_y: i32, max_x: i32, max_y: i32);

    fn has_texture(&self, id: &str) -> bool;

    fn set_scissor(&mut self, pos: Point, size: Size);

    fn clear_scissor(&mut self);
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
    pub quads: Vec<Vertex>,
    pub texture: String,
    pub centroid: Option<[f32; 2]>,
    pub texture_mag_filter: TextureMagFilter,
    pub texture_min_filter: TextureMinFilter,
    pub kind: DrawListKind,
    pub color_filter: [f32; 4],
    pub color_sec: [f32; 4],
    pub scale: [f32; 2],
    pub color_swap_enabled: bool,
    pub swap_hue: f32,
}

impl Default for DrawList {
    #[inline]
    fn default() -> DrawList {
        DrawList {
            quads: Vec::new(),
            texture: String::new(),
            texture_mag_filter: TextureMagFilter::Linear,
            texture_min_filter: TextureMinFilter::Linear,
            kind: DrawListKind::Sprite,
            centroid: None,
            color_filter: [1.0, 1.0, 1.0, 1.0],
            color_sec: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0],
            color_swap_enabled: false,
            swap_hue: 0.0,
        }
    }
}

impl DrawList {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    /// Creates an empty DrawList.  Attempting to draw an empty DrawList will most
    /// likely result in a panic.  You must use `append` to add vertices to this
    /// list if you intend to draw it.
    #[inline]
    pub fn empty_font() -> DrawList {
        DrawList {
            kind: DrawListKind::Font,
            ..Default::default()
        }
    }

    /// Creates an empty DrawList.  Attempting to draw an empty DrawList will most
    /// likely result in a panic, you must use `append` to add vertices to this list
    /// if you intend to draw it.
    #[inline]
    pub fn empty_sprite() -> DrawList {
        Default::default()
    }

    #[inline]
    pub fn from_font(texture: &str, quads: Vec<Vertex>) -> DrawList {
        DrawList {
            texture: texture.to_string(),
            quads,
            kind: DrawListKind::Font,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_texture_id(id: &str, tex_coords: &[f32; 8], rect: Rect) -> DrawList {
        let x_min = rect.x;
        let y_max = Config::ui_height() as f32 - rect.y;
        let x_max = x_min + rect.w;
        let y_min = y_max - rect.h;
        let tc = tex_coords;

        let centroid = Some([(x_min + x_max) / 2.0, (y_min + y_max) / 2.0]);

        let quads = vec![
            Vertex {
                position: [x_min, y_max],
                tex_coords: [tc[0], tc[1]],
            },
            Vertex {
                position: [x_min, y_min],
                tex_coords: [tc[2], tc[3]],
            },
            Vertex {
                position: [x_max, y_max],
                tex_coords: [tc[4], tc[5]],
            },
            Vertex {
                position: [x_max, y_min],
                tex_coords: [tc[6], tc[7]],
            },
            Vertex {
                position: [x_min, y_min],
                tex_coords: [tc[2], tc[3]],
            },
            Vertex {
                position: [x_max, y_max],
                tex_coords: [tc[4], tc[5]],
            },
        ];

        DrawList {
            texture: id.to_string(),
            quads,
            centroid,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_sprite_f32(sprite: &Rc<Sprite>, rect: Rect) -> DrawList {
        DrawList::from_texture_id(&sprite.sheet_id, &sprite.tex_coords, rect)
    }

    #[inline]
    pub fn from_sprite(sprite: &Rc<Sprite>, rect: Rect) -> DrawList {
        DrawList::from_sprite_f32(sprite, rect)
    }

    #[inline]
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = [scale.x, scale.y];
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.color_filter = [color.r, color.g, color.b, color.a];
    }

    #[inline]
    pub fn set_color_sec(&mut self, color: Color) {
        self.color_sec = [color.r, color.g, color.b, color.a];
    }

    #[inline]
    pub fn set_alpha(&mut self, alpha: f32) {
        self.color_filter = [
            self.color_filter[0],
            self.color_filter[1],
            self.color_filter[2],
            alpha,
        ];
    }

    /// enables color swapping for this draw list and sets the hue that the
    /// swap hue will be swapped to
    #[inline]
    pub fn set_swap_hue(&mut self, hue: f32) {
        self.color_swap_enabled = true;
        self.swap_hue = hue;
    }

    /// appends the contents of the other drawlist to this one, moving
    /// the vertex data out into this DrawList's vertex data.
    #[inline]
    pub fn append(&mut self, other: &mut DrawList) {
        self.quads.append(&mut other.quads);
        if self.texture.is_empty() {
            self.texture = other.texture.to_string();
        }
        if self.centroid.is_none() {
            if let Some(centroid) = other.centroid {
                self.centroid = Some(centroid);
            }
        }
    }

    /// rotates the vertices in this drawlist by the given angle,
    /// about the center of the drawlist.  this
    /// is done in software, prior to sending the vertices to the GPU,
    /// so it should be used sparingly
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        let sin = angle.sin();
        let cos = angle.cos();

        let (avg_x, avg_y) = match self.centroid {
            None => {
                let mut avg_x = 0.0;
                let mut avg_y = 0.0;

                for vertex in self.quads.iter() {
                    avg_x += vertex.position[0];
                    avg_y += vertex.position[1];
                }

                avg_x /= self.quads.len() as f32;
                avg_y /= self.quads.len() as f32;
                (avg_x, avg_y)
            }
            Some(centroid) => (centroid[0], centroid[1]),
        };

        for vertex in self.quads.iter_mut() {
            let x = vertex.position[0] - avg_x;
            let y = vertex.position[1] - avg_y;

            let new_x = x * cos - y * sin;
            let new_y = x * sin + y * cos;

            vertex.position[0] = new_x + avg_x;
            vertex.position[1] = -new_y + avg_y;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub enum System {
    Glium(glium_adapter::GliumSystem),
}

impl System {
    pub fn create() -> Result<System, Error> {
        // just always create glium for now
        let glium_system = glium_adapter::create_system()?;

        Ok(System::Glium(glium_system))
    }

    pub fn main_loop(self, updater: Box<dyn ControlFlowUpdater>) {
        match self {
            System::Glium(glium_system) => {
                glium_adapter::main_loop(glium_system, updater);
            }
        }
    }

    pub fn get_display_configurations(&self) -> Vec<DisplayConfiguration> {
        match self {
            System::Glium(glium_system) => glium_system
                .io
                .get_display_configurations(&glium_system.event_loop),
        }
    }
}

const RESOLUTIONS: [(u32, u32); 8] = [
    (3840, 2160),
    (2560, 1440),
    (1920, 1080),
    (1768, 992),
    (1600, 900),
    (1536, 864),
    (1366, 768),
    (1280, 720),
];
