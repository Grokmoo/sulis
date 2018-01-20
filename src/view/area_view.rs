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

use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use grt::ui::{color, Cursor, WidgetKind, Widget};
use grt::io::*;
use grt::io::event::ClickKind;
use grt::util::Point;

use extern_image::{ImageBuffer, Rgba};
use module::area::Layer;
use view::{ActionMenu, EntityMouseover};
use state::GameState;

pub struct AreaView {
    mouse_over: Rc<RefCell<Widget>>,
    scale: RefCell<(f32, f32)>,
    cursors: RefCell<Option<DrawList>>,
    cache_invalid: RefCell<bool>,

    layers: RefCell<Vec<(String, ImageBuffer<Rgba<u8>, Vec<u8>>)>>,
}

const TILE_CACHE_TEXTURE_SIZE: u32 = 1024;
const TILE_SIZE: u32 = 16;
const TEX_COORDS: [f32; 8] = [ 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0 ];

impl AreaView {
    pub fn new(mouse_over: Rc<RefCell<Widget>>) -> Rc<AreaView> {
        Rc::new(AreaView {
            mouse_over: mouse_over,
            scale: RefCell::new((1.0, 1.0)),
            cursors: RefCell::new(None),
            cache_invalid: RefCell::new(true),
            layers: RefCell::new(Vec::new()),
        })
    }

    pub fn clear_cursors(&self) {
        *self.cursors.borrow_mut() = None;
    }

    pub fn add_cursor(&self, mut cursor: DrawList) {
        match *self.cursors.borrow_mut() {
            Some(ref mut c) => {
                c.append(&mut cursor);
                return;
            },
            None => {},
        };

        *self.cursors.borrow_mut() = Some(cursor);
    }

    fn get_cursor_pos_no_scroll(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        self.get_cursor_pos_scaled(widget.borrow().state.position.x,
            widget.borrow().state.position.y)
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        let pos = widget.borrow().state.position;
        let (x, y) = self.get_cursor_pos_scaled(pos.x, pos.y);
        (x + widget.borrow().state.scroll_pos.x, y + widget.borrow().state.scroll_pos.y)
    }

    fn get_cursor_pos_scaled(&self, pos_x: i32, pos_y: i32) -> (i32, i32) {
        let mut x = Cursor::get_x_f32() - pos_x as f32;
        let mut y = Cursor::get_y_f32() - pos_y as f32;

        let (scale_x, scale_y) = *self.scale.borrow();
        x = x / scale_x;
        y = y / scale_y;

        (x as i32, y as i32)
    }

    fn draw_tiles_to_buffer(&self, layer: &Layer, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let max_tile_x = cmp::min((buffer.width() / TILE_SIZE) as i32, layer.width);
        let max_tile_y = cmp::min((buffer.height() / TILE_SIZE) as i32, layer.height);
        let spritesheet = layer.get_spritesheet();
        let source = &spritesheet.image;

        for tile_y in 0..max_tile_y {
            for tile_x in 0..max_tile_x {
                let dest_x = TILE_SIZE * tile_x as u32;
                let dest_y = TILE_SIZE * tile_y as u32;

                let sprite = match layer.image_at(tile_x, tile_y) {
                    &None => continue,
                    &Some(ref image) => image,
                };

                let src_x = sprite.position.x as u32;
                let src_y = sprite.position.y as u32;
                for y in 0..sprite.size.height {
                    for x in 0..sprite.size.width {
                        buffer.put_pixel(dest_x + x as u32, dest_y + y as u32,
                                         *source.get_pixel(src_x + x as u32, src_y + y as u32));
                    }
                }
            }
        }
    }

    fn draw_layer(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                  widget: &Widget, id: &String) {
        let p = widget.state.inner_position;
        let s = widget.state.scroll_pos;
        let mut draw_list =
            DrawList::from_texture_id(&id, &TEX_COORDS,
                                      (p.x - s.x) as f32,
                                      (p.y - s.y) as f32,
                                      (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32,
                                      (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32);
        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);
    }
}

impl WidgetKind for AreaView {
    fn get_name(&self) -> &str {
        "area"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.clear_cursors();
        let area_state = GameState::area_state();

        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;
        widget.borrow_mut().state.set_max_scroll_pos(width, height);
        self.mouse_over.borrow_mut().state.add_text_arg("0", "");
        self.mouse_over.borrow_mut().state.add_text_arg("1", "");

        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;

        trace!("Rendering area tiles to texture for '{}'", area.id);
        for layer in area.terrain.layers.iter() {
            let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE, TILE_CACHE_TEXTURE_SIZE);
            self.draw_tiles_to_buffer(&layer, &mut buffer);
            let id = format!("_{}_{}", area.id, layer.id);
            self.layers.borrow_mut().push((id, buffer));
        }
        *self.cache_invalid.borrow_mut() = true;

        Vec::with_capacity(0)
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, _millis: u32) {
        let scale_x = 1600.0 / (pixel_size.x as f32);
        let scale_y = 900.0 / (pixel_size.y as f32);
        *self.scale.borrow_mut() = (scale_x, scale_y);

        let area_state = GameState::area_state();
        let state = area_state.borrow();
        let ref area = state.area;

        if *self.cache_invalid.borrow() {
            for &(ref id, ref layer) in self.layers.borrow().iter() {
                if !renderer.has_texture(&id) {
                    trace!("Register cached tiles texture for '{}'", &id);
                    renderer.register_texture(&id, layer.clone(),
                        TextureMinFilter::Nearest, TextureMagFilter::Nearest);
                }
            }
            *self.cache_invalid.borrow_mut() = false;
        }

        let p = widget.state.inner_position;
        let num_layers = self.layers.borrow().len();
        let mut layer_index = 0;

        while layer_index <= area.terrain.entity_layer_index {
            self.draw_layer(renderer, scale_x, scale_y, widget,
                            &self.layers.borrow()[layer_index].0);
            layer_index += 1;
        }

        let mut draw_list = DrawList::empty_sprite();
        draw_list.set_scale(scale_x, scale_y);
        draw_list.texture_mag_filter = TextureMagFilter::Nearest;

        for transition in area.transitions.iter() {
            draw_list.append(&mut DrawList::from_sprite(
                    &transition.image_display,
                    transition.from.x + p.x - widget.state.scroll_pos.x,
                    transition.from.y + p.y - widget.state.scroll_pos.y,
                    transition.size.width, transition.size.height));
        }

        for entity in state.entity_iter() {
            let entity = entity.borrow();
            let size = entity.size() as f32;
            let x = (entity.location.x + p.x - widget.state.scroll_pos.x) as f32 + entity.sub_pos.0;
            let y = (entity.location.y + p.y - widget.state.scroll_pos.y) as f32 + entity.sub_pos.1;
            draw_list.append(&mut DrawList::from_sprite_f32(
                    &entity.actor.actor.image_display, x, y, size, size));
        }

        renderer.draw(draw_list);

        while layer_index < num_layers {
            self.draw_layer(renderer, scale_x, scale_y, widget,
                            &self.layers.borrow()[layer_index].0);
            layer_index += 1;
        }

        if let Some(ref cursor) = *self.cursors.borrow() {
            let mut draw_list = cursor.clone();
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        GameState::draw_graphics_mode(renderer, pixel_size);
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        let area_state = GameState::area_state();
        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;
        let (scale_x, scale_y) = *self.scale.borrow();
        widget.borrow_mut().state.set_max_scroll_pos((width as f32 * scale_x).ceil() as i32
                                                     , (height as f32 * scale_y).ceil() as i32);

        use grt::io::InputAction::*;
        match key {
           ScrollUp => widget.borrow_mut().state.scroll(0, -1),
           ScrollDown => widget.borrow_mut().state.scroll(0, 1),
           ScrollLeft => widget.borrow_mut().state.scroll(-1, 0),
           ScrollRight => widget.borrow_mut().state.scroll(1, 0),
           _ => return false,
        };
        true
    }


    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let (x, y) = self.get_cursor_pos(widget);
        if x < 0 || y < 0 { return true; }

        let area_state = GameState::area_state();
        let action_menu = ActionMenu::new(x, y);
        if kind == ClickKind::Left {
            action_menu.fire_default_callback();

            if let Some(entity) = area_state.borrow().get_entity_at(x, y) {
                Widget::set_mouse_over(widget, EntityMouseover::new(&entity));
            }

        } else if kind == ClickKind::Right {
            Widget::add_child_to(widget, Widget::with_defaults(action_menu));
        }

        true
    }

    fn on_mouse_move(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        let (area_x, area_y) = self.get_cursor_pos(widget);
        let area_state = GameState::area_state();

        {
            let ref mut state = self.mouse_over.borrow_mut().state;
            state.clear_text_args();
            state.add_text_arg("0", &format!("{}", area_x));
            state.add_text_arg("1", &format!("{}", area_y));
        }
        self.mouse_over.borrow_mut().invalidate_layout();
        self.clear_cursors();

        if let Some(entity) = area_state.borrow().get_entity_at(area_x, area_y) {
            Widget::set_mouse_over(widget, EntityMouseover::new(&entity));

            let pc = GameState::pc();
            if *pc.borrow() != *entity.borrow() {
                let sprite = &entity.borrow().size.cursor_sprite;
                let x = entity.borrow().location.x;
                let y = entity.borrow().location.y;
                let size = entity.borrow().size();

                let mut cursor = DrawList::from_sprite(sprite, x, y, size, size);
                cursor.set_color(color::RED);
                self.add_cursor(cursor);
            }
        }

        let pc = GameState::pc();
        let size = pc.borrow().size();

        let (c_x, c_y) = self.get_cursor_pos_no_scroll(widget);
        let mut draw_list = DrawList::from_sprite(&pc.borrow().size.cursor_sprite,
        c_x - size / 2, c_y - size / 2, size, size);

        let action_menu = ActionMenu::new(area_x, area_y);
        if !action_menu.is_default_callback_valid() {
            draw_list.set_color(color::RED);
        }

        self.add_cursor(draw_list);

        true
    }

    fn on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.mouse_over.borrow_mut().state.clear_text_args();
        self.clear_cursors();
        true
    }
}
