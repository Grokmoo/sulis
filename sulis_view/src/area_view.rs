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

use std::any::Any;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::cmp;
use std::time;

use sulis_core::ui::animation_state;
use sulis_core::ui::{color, Cursor, WidgetKind, Widget};
use sulis_core::io::*;
use sulis_core::io::event::ClickKind;
use sulis_core::util::{self, Point};
use sulis_core::config::CONFIG;
use sulis_core::resource::Sprite;
use sulis_core::extern_image::ImageBuffer;
use sulis_module::area::Layer;
use sulis_state::{AreaState, GameState};

use {ActionMenu, EntityMouseover, PropMouseover};

const NAME: &'static str = "area";

pub struct AreaView {
    mouse_over: Rc<RefCell<Widget>>,
    scale: (f32, f32),
    cursors: Option<DrawList>,
    cache_invalid: bool,
    layers: Vec<String>,

    scroll_x_f32: f32,
    scroll_y_f32: f32,
}

const SCALE_Y_BASE: f32 = 3200.0;
const SCALE_X_BASE: f32 = SCALE_Y_BASE * 16.0 / 9.0;
const TILE_CACHE_TEXTURE_SIZE: u32 = 2048;
const TILE_SIZE: u32 = 16;
const TEX_COORDS: [f32; 8] = [ 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0 ];

const VISIBILITY_TEX_ID: &'static str = "__visibility__";

impl AreaView {
    pub fn new(mouse_over: Rc<RefCell<Widget>>) -> Rc<RefCell<AreaView>> {
        Rc::new(RefCell::new(AreaView {
            mouse_over: mouse_over,
            scale: (1.0, 1.0),
            cursors: None,
            cache_invalid: true,
            layers: Vec::new(),
            scroll_x_f32: 0.0,
            scroll_y_f32: 0.0,
        }))
    }

    pub fn get_mouseover_pos(&self, widget: &Rc<RefCell<Widget>>,
                             x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
        let x = x as f32 + width as f32 / 2.0;
        let y = y as f32 + height as f32;
        let x = ((x - widget.borrow().state.scroll_pos.x as f32) * self.scale.0).round() as i32;
        let y = ((y - widget.borrow().state.scroll_pos.y as f32) * self.scale.1).round() as i32;

        (x, y)
    }

    pub fn clear_cursors(&mut self) {
        self.cursors = None;
    }

    pub fn add_cursor(&mut self, mut cursor: DrawList) {
        match self.cursors {
            Some(ref mut c) => {
                c.append(&mut cursor);
                return;
            },
            None => {},
        };

        self.cursors = Some(cursor);
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

        let (scale_x, scale_y) = self.scale;
        x = x / scale_x;
        y = y / scale_y;

        (x as i32, y as i32)
    }

    fn draw_layer_to_texture(&self, renderer: &mut GraphicsRenderer, layer: &Layer) {
        let (max_tile_x, max_tile_y) = AreaView::get_texture_cache_max(layer.width, layer.height);
        let mut draw_list = DrawList::empty_sprite();

        for tile_y in 0..max_tile_y {
            for tile_x in 0..max_tile_x {
                let tile = match layer.tile_at(tile_x, tile_y) {
                    &None => continue,
                    &Some(ref tile) => tile,
                };
                let sprite = &tile.image_display;

                draw_list.append(&mut DrawList::from_sprite(sprite, tile_x, tile_y,
                                                            tile.width, tile.height));
            }
        }

        AreaView::draw_list_to_texture(renderer, draw_list, &layer.id);
    }

    fn draw_visibility_to_texture(&self, renderer: &mut GraphicsRenderer, sprite: &Rc<Sprite>,
                                  area_state: &RefMut<AreaState>) {
        let start_time = time::Instant::now();
        renderer.clear_texture(VISIBILITY_TEX_ID);
        let (max_tile_x, max_tile_y) = AreaView::get_texture_cache_max(area_state.area.width,
                                                                       area_state.area.height);

        let mut draw_list = DrawList::empty_sprite();

        for tile_y in 0..max_tile_y {
            for tile_x in 0..max_tile_x {
                if area_state.is_pc_visible(tile_x, tile_y) { continue; }
                draw_list.append(&mut DrawList::from_sprite(sprite, tile_x, tile_y, 1, 1));
            }
        }

        AreaView::draw_list_to_texture(renderer, draw_list, VISIBILITY_TEX_ID);
        trace!("Visibility render to texture time: {}",
              util::format_elapsed_secs(start_time.elapsed()));
    }

    fn draw_list_to_texture(renderer: &mut GraphicsRenderer, draw_list: DrawList, texture_id: &str) {
        renderer.clear_texture(texture_id);
        let mut draw_list = draw_list;
        draw_list.texture_mag_filter = TextureMagFilter::Linear;
        draw_list.texture_min_filter = TextureMinFilter::Linear;
        draw_list.set_scale(TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 *
                            CONFIG.display.width as f32,
                            TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 *
                            CONFIG.display.height as f32);
        renderer.draw_to_texture(texture_id, draw_list);
    }

    fn get_texture_cache_max(width: i32, height: i32) -> (i32, i32) {
        let x = cmp::min((TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as i32, width);
        let y = cmp::min((TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as i32, height);

        (x, y)
    }

    fn draw_layer(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                  widget: &Widget, id: &str) {
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

    fn recompute_max_scroll(&mut self, widget: &Rc<RefCell<Widget>>) {
        let area_state = GameState::area_state();
        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;
        let (scale_x, scale_y) = self.scale;
        // TODO fix this
        widget.borrow_mut().state.set_max_scroll_pos((width as f32 * scale_x).ceil() as i32
                                                     , (height as f32 * scale_y).ceil() as i32);
    }

    fn draw_entities(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                     _alpha: f32, widget: &Widget, state: &AreaState, millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.scroll_pos;

        let mut draw_list = DrawList::empty_sprite();
        draw_list.set_scale(scale_x, scale_y);
        for prop_state in state.prop_iter() {
            let x = (prop_state.location.x + p.x - s.x) as f32;
            let y = (prop_state.location.y + p.y - s.y) as f32;
            prop_state.append_to_draw_list(&mut draw_list, x, y, millis);
        }

        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        for entity in state.entity_iter() {
            let entity = entity.borrow();

            if entity.location_points().any(|p| state.is_pc_visible(p.x, p.y)) {
                let x = (entity.location.x + p.x - s.x) as f32 + entity.sub_pos.0;
                let y = (entity.location.y + p.y - s.y) as f32 + entity.sub_pos.1;

                // TODO implement drawing with alpha
                entity.actor.draw_graphics_mode(renderer, scale_x, scale_y,
                                                x, y, millis);
            }
        }
    }
}

impl WidgetKind for AreaView {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.clear_cursors();
        let area_state = GameState::area_state();

        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;
        widget.borrow_mut().state.set_max_scroll_pos(width, height);
        self.mouse_over.borrow_mut().state.add_text_arg("0", "");
        self.mouse_over.borrow_mut().state.add_text_arg("1", "");

        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;

        for layer in area.terrain.layers.iter() {
            self.layers.push(layer.id.to_string());
        }
        self.cache_invalid = true;

        Vec::with_capacity(0)
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        let scale_x = SCALE_X_BASE / (pixel_size.x as f32);
        let scale_y = SCALE_Y_BASE / (pixel_size.y as f32);
        self.scale = (scale_x, scale_y);

        let area_state = GameState::area_state();
        let mut state = area_state.borrow_mut();

        if self.cache_invalid {
            debug!("Caching area '{}' layers to texture", state.area.id);

            if !renderer.has_texture(VISIBILITY_TEX_ID) {
                renderer.register_texture(VISIBILITY_TEX_ID,
                                          ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE, TILE_CACHE_TEXTURE_SIZE),
                                          TextureMinFilter::Nearest,
                                          TextureMagFilter::Nearest);
            }

            for layer in state.area.terrain.layers.iter() {
                let id = &layer.id;
                trace!("Caching layer '{}'", id);
                if !renderer.has_texture(&id) {
                    renderer.register_texture(&id,
                                              ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE,
                                                               TILE_CACHE_TEXTURE_SIZE),
                                              TextureMinFilter::Nearest,
                                              TextureMagFilter::Nearest);
                }

                self.draw_layer_to_texture(renderer, &layer);
            }

            self.cache_invalid = false;
        }

        if state.pc_vis_cache_invalid {
            trace!("Redrawing PC visibility to texture");
            self.draw_visibility_to_texture(renderer, &state.area.visibility_tile, &state);
            state.pc_vis_cache_invalid = false;
        }

        let p = widget.state.inner_position;
        let s = widget.state.scroll_pos;
        let num_layers = self.layers.len();
        let mut layer_index = 0;

        while layer_index <= state.area.terrain.entity_layer_index {
            self.draw_layer(renderer, scale_x, scale_y, widget,
                            &self.layers[layer_index]);
            layer_index += 1;
        }


        let mut draw_list = DrawList::empty_sprite();
        for transition in state.area.transitions.iter() {
            draw_list.set_scale(scale_x, scale_y);
            transition.image_display.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                                        (transition.from.x + p.x - s.x) as f32,
                                                        (transition.from.y + p.y - s.y) as f32,
                                                        transition.size.width as f32,
                                                        transition.size.height as f32,
                                                        millis);
        }

        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        self.draw_entities(renderer, scale_x, scale_y, 1.0, widget, &state, millis);

        while layer_index < num_layers {
            self.draw_layer(renderer, scale_x, scale_y, widget,
                            &self.layers[layer_index]);
            layer_index += 1;
        }

        self.draw_layer(renderer, scale_x, scale_y, widget, VISIBILITY_TEX_ID);

        //draw transparent version of each actor for when they are obscured behind objects
        // self.draw_entities(renderer, scale_x, scale_y, 0.4, widget, &state, millis);

        if let Some(ref cursor) = self.cursors {
            let mut draw_list = cursor.clone();
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        for feedback_text in state.feedback_text_iter() {
            feedback_text.draw(renderer, p.x - s.x, p.y - s.y, scale_x, scale_y);
        }


        GameState::draw_graphics_mode(renderer, p.x - s.x, p.y - s.y,
                                      scale_x, scale_y, millis);
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        self.recompute_max_scroll(widget);

        use sulis_core::io::InputAction::*;
        match key {
           ScrollUp => widget.borrow_mut().state.scroll(0, -1),
           ScrollDown => widget.borrow_mut().state.scroll(0, 1),
           ScrollLeft => widget.borrow_mut().state.scroll(-1, 0),
           ScrollRight => widget.borrow_mut().state.scroll(1, 0),
           _ => return false,
        };
        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let (x, y) = self.get_cursor_pos(widget);
        if x < 0 || y < 0 { return true; }

        let area_state = GameState::area_state();
        let action_menu = ActionMenu::new(x, y);
        if kind == ClickKind::Left {
            action_menu.borrow().fire_default_callback(&widget, self);

            if let Some(entity) = area_state.borrow().get_entity_at(x, y) {
                let (x, y) = {
                    let entity = entity.borrow();
                    self.get_mouseover_pos(widget, entity.location.x, entity.location.y,
                        entity.size.size, entity.size.size)
                };
                Widget::set_mouse_over(widget, EntityMouseover::new(&entity), x, y);
            }

        } else if kind == ClickKind::Right {
            Widget::add_child_to(widget, Widget::with_defaults(action_menu));
        }

        true
    }

    fn on_mouse_drag(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind,
                     delta_x: f32, delta_y: f32) -> bool {
        self.recompute_max_scroll(widget);

        match kind {
            ClickKind::Middle => {
                let max_x = widget.borrow().state.max_scroll_pos.x as f32;
                let max_y = widget.borrow().state.max_scroll_pos.y as f32;

                self.scroll_x_f32 -= delta_x;
                self.scroll_y_f32 -= delta_y;
                if self.scroll_x_f32 < 0.0 { self.scroll_x_f32 = 0.0; }
                if self.scroll_y_f32 < 0.0 { self.scroll_y_f32 = 0.0; }
                if self.scroll_x_f32 > max_x { self.scroll_x_f32 = max_x; }
                if self.scroll_y_f32 > max_y { self.scroll_y_f32 = max_y; }

                widget.borrow_mut().state.set_scroll(self.scroll_x_f32 as i32,
                                                     self.scroll_y_f32 as i32);
           },
            _ => (),
        }

        true
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
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
            let (x, y) = {
                let entity = entity.borrow();
                self.get_mouseover_pos(widget, entity.location.x, entity.location.y,
                    entity.size.size, entity.size.size)
            };
            Widget::set_mouse_over(widget, EntityMouseover::new(&entity), x, y);

            let pc = GameState::pc();
            if *pc.borrow() != *entity.borrow() {
                let sprite = &entity.borrow().size.cursor_sprite;
                let x = entity.borrow().location.x - widget.borrow().state.scroll_pos.x;
                let y = entity.borrow().location.y - widget.borrow().state.scroll_pos.y;
                let size = entity.borrow().size();

                let mut cursor = DrawList::from_sprite(sprite, x, y, size, size);
                cursor.set_color(color::RED);
                self.add_cursor(cursor);
            }
        } else if let Some(index) = area_state.borrow().prop_index_at(area_x, area_y) {
            let (x, y) = {
                let prop_state = &area_state.borrow().props[index];
                self.get_mouseover_pos(widget, prop_state.location.x, prop_state.location.y,
                                       prop_state.prop.width as i32, prop_state.prop.height as i32)
            };
            Widget::set_mouse_over(widget, PropMouseover::new(index), x, y);
        }

        let pc = GameState::pc();
        let size = pc.borrow().size();

        let (c_x, c_y) = self.get_cursor_pos_no_scroll(widget);
        let mut draw_list = DrawList::from_sprite(&pc.borrow().size.cursor_sprite,
        c_x - size / 2, c_y - size / 2, size, size);

        let action_menu = ActionMenu::new(area_x, area_y);
        if !action_menu.borrow().is_default_callback_valid() {
            draw_list.set_color(color::RED);
        }

        self.add_cursor(draw_list);

        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.mouse_over.borrow_mut().state.clear_text_args();
        self.clear_cursors();
        true
    }
}
