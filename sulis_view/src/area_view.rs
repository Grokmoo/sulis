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
use std::mem;

use sulis_core::image::Image;
use sulis_core::ui::{compute_area_scaling, animation_state};
use sulis_core::ui::{color, Cursor, Scrollable, WidgetKind, Widget};
use sulis_core::io::*;
use sulis_core::io::event::ClickKind;
use sulis_core::util::{self, Point};
use sulis_core::config::Config;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::extern_image::ImageBuffer;
use sulis_widgets::Label;
use sulis_module::{area::{Layer, Tile}};
use sulis_state::{AreaDrawable, AreaState, EntityState, EntityTextureCache, GameState};
use sulis_state::area_feedback_text;

use {AreaMouseover, action_kind};

struct HoverSprite {
    pub sprite: Rc<Sprite>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub left_click_action_valid: bool,
}

const NAME: &'static str = "area";

pub struct AreaView {
    scale: (f32, f32),
    cache_invalid: bool,
    layers: Vec<String>,
    entity_texture_cache: EntityTextureCache,

    targeter_label: Rc<RefCell<Widget>>,
    targeter_tile: Option<Rc<Image>>,
    selection_box_image: Option<Rc<Image>>,

    scroll: Scrollable,
    hover_sprite: Option<HoverSprite>,
    selection_box_start: Option<(f32, f32)>,
    feedback_text_params: area_feedback_text::Params,

    area_mouseover: Option<Rc<RefCell<AreaMouseover>>>,
    area_mouseover_widget: Option<Rc<RefCell<Widget>>>,

    scroll_target: Option<(f32, f32)>,
}

const TILE_CACHE_TEXTURE_SIZE: u32 = 2048;
const TILE_SIZE: u32 = 16;
const TEX_COORDS: [f32; 8] = [ 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0 ];

const ENTITY_TEX_ID: &'static str = "__entities__";
const VISIBILITY_TEX_ID: &'static str = "__visibility__";
const BASE_LAYER_ID: &str = "__base_layer__";
const AERIAL_LAYER_ID: &str = "__aerial_layer__";

impl AreaView {
    pub fn new() -> Rc<RefCell<AreaView>> {
        Rc::new(RefCell::new(AreaView {
            targeter_label: Widget::with_theme(Label::empty(), "targeter_label"),
            scale: (1.0, 1.0),
            hover_sprite: None,
            cache_invalid: true,
            entity_texture_cache: EntityTextureCache::new(ENTITY_TEX_ID, TILE_CACHE_TEXTURE_SIZE, TILE_SIZE),
            layers: Vec::new(),
            scroll: Scrollable::new(),
            targeter_tile: None,
            selection_box_image: None,
            selection_box_start: None,
            feedback_text_params: area_feedback_text::Params::default(),
            area_mouseover: None,
            area_mouseover_widget: None,
            scroll_target: None,
        }))
    }

    pub fn center_scroll_on(&mut self, entity: &Rc<RefCell<EntityState>>, area_width: i32,
                            area_height: i32, widget: &Widget) {
        let x = entity.borrow().location.x as f32 + entity.borrow().size.width as f32 / 2.0;
        let y = entity.borrow().location.y as f32 + entity.borrow().size.height as f32 / 2.0;

        let (x, y) = self.center_scroll_on_point(x, y, area_width, area_height, widget);
        self.scroll.set(x, y);
    }

    fn center_scroll_on_point(&mut self, x: f32, y: f32, area_width: i32,
                              area_height: i32, widget: &Widget) -> (f32, f32) {
        let (scale_x, scale_y) = self.scale;
        self.scroll.compute_max(widget, area_width, area_height, scale_x, scale_y);

        let x = x - widget.state.inner_width() as f32 / scale_x / 2.0;
        let y = y - widget.state.inner_height() as f32 / scale_y / 2.0;

        (x, y)
    }

    pub fn delayed_scroll_to_point(&mut self, x: f32, y: f32, area_width: i32,
                                   area_height: i32, widget: &Widget) {
        let (x, y) = self.center_scroll_on_point(x, y, area_width, area_height, widget);
        let (x, y) = self.scroll.bound(x, y);
        self.scroll_target = Some((x, y));
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        let pos = widget.borrow().state.inner_position;
        let (x, y) = self.get_cursor_pos_scaled(pos.x, pos.y);
        ((x + self.scroll.x()) as i32, (y + self.scroll.y()) as i32)
    }

    fn get_cursor_pos_scaled(&self, pos_x: i32, pos_y: i32) -> (f32, f32) {
        let mut x = Cursor::get_x_f32() - pos_x as f32;
        let mut y = Cursor::get_y_f32() - pos_y as f32;

        let (scale_x, scale_y) = self.scale;
        x = x / scale_x;
        y = y / scale_y;

        (x, y)
    }

    fn draw_layer_to_texture(&self, renderer: &mut GraphicsRenderer, layer: &Layer, texture_id: &str) {
        let (max_tile_x, max_tile_y) = AreaView::get_texture_cache_max(layer.width, layer.height);

        let mut tiles: Vec<(i32, i32, Rc<Tile>)> = Vec::new();
        for tile_y in 0..max_tile_y {
            for tile_x in 0..max_tile_x {
                for tile in layer.tiles_at(tile_x, tile_y) {
                    tiles.push((tile_x, tile_y, Rc::clone(tile)));
                }
            }
        }

        // sort by the bottom y coordinate - use a stable sort so tiles maintain
        // their ordering otherwise
        tiles.sort_by(|a, b| { (a.1 + a.2.height).cmp(&(b.1 + b.2.height)) });

        let mut draw_list = DrawList::empty_sprite();
        for (x, y, tile) in tiles {
            draw_list.append(&mut DrawList::from_sprite(&tile.image_display, x, y,
                                                        tile.width, tile.height));
        }

        if !draw_list.is_empty() {
            AreaView::draw_list_to_texture(renderer, draw_list, texture_id);
        }
    }

    fn draw_visibility_to_texture(&self, renderer: &mut GraphicsRenderer, vis_sprite: &Rc<Sprite>,
                                  explored_sprite: &Rc<Sprite>, area_state: &RefMut<AreaState>,
                                  delta_x: i32, delta_y: i32) {
        let start_time = time::Instant::now();
        let (max_tile_x, max_tile_y) = AreaView::get_texture_cache_max(area_state.area.width,
                                                                       area_state.area.height);

        let vis_dist = area_state.area.vis_dist;
        for pc in GameState::party() {
            let c_x = pc.borrow().location.x + pc.borrow().size.width / 2;
            let c_y = pc.borrow().location.y + pc.borrow().size.height / 2;
            let min_x = cmp::max(0, c_x - vis_dist + if delta_x < 0 { delta_x } else { 0 });
            let max_x = cmp::min(max_tile_x, 1 + c_x + vis_dist + if delta_x > 0 { delta_x } else { 0 });
            let min_y = cmp::max(0, c_y - vis_dist + if delta_y < 0 { delta_y } else { 0 });
            let max_y = cmp::min(max_tile_y, 1 + c_y + vis_dist + if delta_y > 0 { delta_y } else { 0 });

            let scale = TILE_SIZE as i32;
            renderer.clear_texture_region(VISIBILITY_TEX_ID, min_x * scale, min_y * scale,
                                          max_x * scale, max_y * scale);
            self.draw_vis_to_texture(renderer, vis_sprite, explored_sprite, area_state,
                                     min_x, min_y, max_x, max_y);
            trace!("Visibility render to texture time: {}",
                   util::format_elapsed_secs(start_time.elapsed()));
        }
    }

    fn draw_vis_to_texture(&self, renderer: &mut GraphicsRenderer, vis_sprite: &Rc<Sprite>,
                           explored_sprite: &Rc<Sprite>, area_state: &RefMut<AreaState>,
                           min_x: i32, min_y: i32, max_x: i32, max_y: i32) {
        let mut draw_list = DrawList::empty_sprite();

        // info!("======");
        for tile_y in min_y..max_y {
            // let mut cur_line = "".to_string();
            for tile_x in min_x..max_x {
                if area_state.is_pc_visible(tile_x, tile_y) {
                    // cur_line.push('x');
                    continue;
                } else {
                    // cur_line.push(' ');
                }
                draw_list.append(&mut DrawList::from_sprite(vis_sprite, tile_x, tile_y, 1, 1));

                if area_state.is_pc_explored(tile_x, tile_y) { continue; }
                draw_list.append(&mut DrawList::from_sprite(explored_sprite, tile_x, tile_y, 1, 1));
            }
            // info!("{}|", cur_line);
        }

        AreaView::draw_list_to_texture(renderer, draw_list, VISIBILITY_TEX_ID);
    }

    fn draw_list_to_texture(renderer: &mut GraphicsRenderer, draw_list: DrawList, texture_id: &str) {
        let (ui_x, ui_y) = Config::ui_size();
        let mut draw_list = draw_list;
        draw_list.texture_mag_filter = TextureMagFilter::Linear;
        draw_list.texture_min_filter = TextureMinFilter::Linear;
        draw_list.set_scale(TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 * ui_x as f32,
                            TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 * ui_y as f32);
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
        let mut draw_list =
            DrawList::from_texture_id(&id, &TEX_COORDS,
                                      p.x as f32 - self.scroll.x(),
                                      p.y as f32 - self.scroll.y(),
                                      (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32,
                                      (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32);
        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);
    }

    fn draw_entities_props(&mut self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
                     alpha: f32, widget: &Widget, state: &AreaState, millis: u32) {
        // let start_time = time::Instant::now();
        let mut to_draw: Vec<&AreaDrawable> = Vec::new();

        for prop_state in state.prop_iter() {
            to_draw.push(&*prop_state);
        }

        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        for index in state.entity_iter() {
            let entity = mgr.entity(*index);
            let mut entity = entity.borrow_mut();
            if !entity.location_points().any(|p| state.is_pc_visible(p.x, p.y)) {
                continue;
            }

            entity.cache(renderer, &mut self.entity_texture_cache);

            let entity = unsafe {
                mem::transmute::<&EntityState, &'static EntityState>(&*entity)
            };

            to_draw.push(entity);
        }

        to_draw.sort_by_key(|k| k.location());

        for drawable in to_draw {
            let x = widget.state.inner_position.x as f32 - self.scroll.x();
            let y = widget.state.inner_position.y as f32 - self.scroll.y();
            drawable.draw(renderer, scale_x, scale_y, x, y, millis, alpha);
        }

        // info!("Entity & Prop draw time: {}", util::format_elapsed_secs(start_time.elapsed()));
    }

    fn draw_selection(&mut self, selected: &Rc<RefCell<EntityState>>, renderer: &mut GraphicsRenderer,
                      scale_x: f32, scale_y: f32, widget: &Widget, millis: u32) {
        let x_base = widget.state.inner_position.x as f32 - self.scroll.x();
        let y_base = widget.state.inner_position.y as f32 - self.scroll.y();

        let selected = selected.borrow();
        let w = selected.size.width as f32;
        let h = selected.size.height as f32;
        let x = x_base + selected.location.x as f32 + selected.sub_pos.0;
        let y = y_base + selected.location.y as f32 + selected.sub_pos.1;

        let mut draw_list = DrawList::empty_sprite();
        selected.size.selection_image.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                                          x, y, w, h, millis);

        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);
    }

    fn get_selection_box_coords(&self) -> Option<(f32, f32, f32, f32)> {
        if let Some((x, y)) = self.selection_box_start {
            let (x2, y2) = Cursor::get_position_f32();
            let x_start;
            let y_start;
            let x_end;
            let y_end;
            if x > x2 {
                x_start = x2;
                x_end = x;
            } else {
                x_start = x;
                x_end = x2;
            }
            if y > y2 {
                y_start = y2;
                y_end = y;
            } else {
                y_start = y;
                y_end = y2;
            }

            return Some((x_start, y_start, x_end, y_end));
        }

        None
    }

    fn select_party_in_box(&self, widget: &Widget) -> Vec<Rc<RefCell<EntityState>>> {
        let mut party = Vec::new();

        let (x, y, x_end, y_end) = match self.get_selection_box_coords() {
            None => return party,
            Some((x, y, x2, y2)) => (x, y, x2, y2),
        };

        let pos = widget.state.inner_position;
        let x1 = ((x - pos.x as f32) / self.scale.0 + self.scroll.x()) as i32;
        let y1 = ((y - pos.y as f32) / self.scale.1 + self.scroll.y()) as i32;
        let x2 = ((x_end - pos.x as f32) / self.scale.0 + self.scroll.x()) as i32;
        let y2 = ((y_end - pos.y as f32) / self.scale.1 + self.scroll.y()) as i32;

        for entity in GameState::party().iter() {
            let loc = &entity.borrow().location;
            let size = &entity.borrow().size;

            if loc.x >= x2 || x1 >= loc.x + size.width {
                continue;
            } else if loc.y >= y2 || y1 >= loc.y + size.height {
                continue;
            }

            party.push(Rc::clone(entity));
        }

        party
    }

    fn check_mouseover(&self, x: i32, y: i32) -> Option<Rc<RefCell<AreaMouseover>>> {
        let area_state = GameState::area_state();
        let targeter = area_state.borrow_mut().targeter();

        if let Some(ref targeter) = targeter {
            let mut targeter = targeter.borrow_mut();
            let mouse_over = targeter.on_mouse_move(x, y);

            if let Some(ref entity) = mouse_over {
                return Some(AreaMouseover::new_entity(entity));
            } else {
                return None;
            }
        }

        let area_state = area_state.borrow();
        if let Some(entity) = area_state.get_entity_at(x, y) {
            Some(AreaMouseover::new_entity(&entity))
        } else if let Some(_) = self.check_closed_door(x, y, &area_state) {
            None
        } else if let Some(transition) = area_state.get_transition_at(x, y) {
            Some(AreaMouseover::new_transition(&transition.hover_text))
        } else if let Some(index) = area_state.prop_index_at(x, y) {
            let interactive = {
                let prop = area_state.get_prop(index);
                prop.is_container() && prop.is_enabled()
            };

            if interactive {
                Some(AreaMouseover::new_prop(index))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn check_closed_door(&self, x: i32, y: i32,
                       area_state: &AreaState) -> Option<Rc<RefCell<AreaMouseover>>> {
        if let Some(index) = area_state.prop_index_at(x, y) {
            {
                let prop = area_state.get_prop(index);
                if !prop.is_door() || !prop.is_enabled() || prop.is_active() { return None; }
            }

            Some(AreaMouseover::new_prop(index))
        } else {
            None
        }
    }

    fn set_cursor(&mut self, x: i32, y: i32) {
        let action = action_kind::get_action(x, y);
        Cursor::set_cursor_state(action.cursor_state());
        match action.get_hover_info() {
            Some((size, x, y)) => {
                let hover_sprite = HoverSprite {
                    sprite: Rc::clone(&size.cursor_sprite),
                    x,
                    y,
                    w: size.width,
                    h: size.height,
                    left_click_action_valid: true
                };
                self.hover_sprite = Some(hover_sprite);
            }, None => {
                self.hover_sprite = None;
            }
        }
    }

    pub fn update_cursor_and_hover(&mut self, widget: &Rc<RefCell<Widget>>) {
        let area_state = GameState::area_state();

        self.hover_sprite = None;

        if let Some(_) = self.selection_box_start {
            Cursor::set_cursor_state(animation_state::Kind::Normal);
            return;
        }

        let (area_x, area_y) = self.get_cursor_pos(widget);
        match self.check_mouseover(area_x, area_y) {
            None => {
                self.clear_area_mouseover();
            },
            Some(mouseover) => {
                let (clear, set_new) = if let Some(ref cur_mouseover) = self.area_mouseover {
                    if *cur_mouseover.borrow() == *mouseover.borrow() { (false, false) }
                    else { (true, true) }
                } else { (false, true) };

                if clear { self.clear_area_mouseover(); }
                if set_new { self.set_new_mouseover(widget, mouseover); }
            }
        };

        if area_state.borrow_mut().targeter().is_none() {
            self.set_cursor(area_x, area_y);
        }
    }

    fn set_new_mouseover(&mut self, parent: &Rc<RefCell<Widget>>,
                         mouseover: Rc<RefCell<AreaMouseover>>) {
        self.area_mouseover = Some(Rc::clone(&mouseover));
        let widget = Widget::with_defaults(mouseover);
        self.area_mouseover_widget = Some(Rc::clone(&widget));

        let root = Widget::get_root(parent);
        Widget::add_child_to(&root, widget);
    }

    pub fn clear_area_mouseover(&mut self) {
        if let Some(ref widget) = self.area_mouseover_widget {
            widget.borrow_mut().mark_for_removal();

            // prevent from double drawing if we end up creating
            // a new mouseover on this frame
            widget.borrow_mut().state.set_visible(false);
        }

        self.area_mouseover = None;
        self.area_mouseover_widget = None;
    }

    pub fn scroll(&mut self, delta_x: f32, delta_y: f32, millis: u32) {
        let speed = Config::scroll_speed() * millis as f32 / 33.0;
        let delta_x = speed * delta_x / self.scale.0;
        let delta_y = speed * delta_y / self.scale.1;
        self.scroll.change(delta_x, delta_y)
    }
}

impl WidgetKind for AreaView {
    widget_kind!(NAME);

    fn update(&mut self, _widget: &Rc<RefCell<Widget>>, millis: u32) {
        let (dest_x, dest_y) = match self.scroll_target {
            None => return,
            Some((x, y)) => (x, y),
        };

        let (sign_x, sign_y) = ((self.scroll.x() - dest_x).signum(), (self.scroll.y() - dest_y).signum());

        let speed = Config::scroll_speed() * 4.0 * millis as f32 / 33.3;
        let (speed_x, speed_y) = (speed / self.scale.0, speed / self.scale.1);

        let (cur_x, cur_y) = (self.scroll.x(), self.scroll.y());

        let (dir_x, dir_y) = (cur_x - dest_x, cur_y - dest_y);

        let mag = dir_y.hypot(dir_x);
        let (delta_x, delta_y) = (dir_x * speed_x / mag, dir_y * speed_y / mag);

        self.scroll.change(delta_x, delta_y);

        if (self.scroll.x() - dest_x).signum() != sign_x || (self.scroll.y() - dest_y).signum() != sign_y {
            self.scroll.set(dest_x, dest_y);
            self.scroll_target = None;
        }
    }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if let Some(ref theme) = widget.theme {
            if let Some(ref image_id) = theme.custom.get("targeter_tile") {
                self.targeter_tile = ResourceSet::get_image(image_id);
            }

            if let Some(ref image_id) = theme.custom.get("selection_box_image") {
                self.selection_box_image = ResourceSet::get_image(image_id);
            }

            self.feedback_text_params.scale =
                theme.get_custom_or_default("feedback_text_scale", 1.0);

            if let Some(ref font_id) = theme.custom.get("feedback_text_font") {
                self.feedback_text_params.font = match ResourceSet::get_font(font_id) {
                    None => {
                        warn!("Invalid font specified for area feedback text '{}'", font_id);
                        ResourceSet::get_default_font()
                    }, Some(font) => font,
                };
            }

            self.feedback_text_params.info_color =
                theme.get_custom_or_default("feedback_text_info_color", color::LIGHT_GRAY);
            self.feedback_text_params.miss_color =
                theme.get_custom_or_default("feedback_text_miss_color", color::LIGHT_GRAY);
            self.feedback_text_params.hit_color =
                theme.get_custom_or_default("feedback_text_hit_color", color::RED);
            self.feedback_text_params.heal_color =
                theme.get_custom_or_default("feedback_text_heal_color", color::BLUE);
        }

        if self.targeter_tile.is_none() {
            warn!("No targeter tile specified for Areaview");
        }
    }
    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.hover_sprite = None;

        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;

        for layer in area.layer_set.layers.iter() {
            self.layers.push(layer.id.to_string());
        }
        self.cache_invalid = true;

        vec![Rc::clone(&self.targeter_label)]
    }

    fn on_key_press(&mut self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use sulis_core::io::InputAction::*;
        let delta = match key {
            ZoomIn => 0.1,
            ZoomOut => -0.1,
            _ => return false,
        };

        let old_user_scale = GameState::user_zoom();
        GameState::set_user_zoom(old_user_scale + delta);
        let user_scale = GameState::user_zoom();

        // recenter the view based on the scroll change
        let (old_scale_x, old_scale_y) = self.scale;
        self.scale = (old_scale_x / old_user_scale * user_scale,
                      old_scale_y / old_user_scale * user_scale);

        let width = widget.borrow().state.inner_width() as f32;
        let height = widget.borrow().state.inner_height() as f32;

        let x = self.scroll.x() + width / old_scale_x / 2.0;
        let y = self.scroll.y() + height / old_scale_y / 2.0;

        let area_state = GameState::area_state();
        let area_width = area_state.borrow().area.width;
        let area_height = area_state.borrow().area.height;
        self.center_scroll_on_point(x, y, area_width, area_height, &*widget.borrow());
        true
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
            widget: &Widget, millis: u32) {
        let zoom = GameState::user_zoom();
        {
            let (sx, sy) = compute_area_scaling(pixel_size);
            self.scale = (sx * zoom, sy * zoom);
        }

        let (scale_x, scale_y) = self.scale;

        let area_state = GameState::area_state();
        let mut state = area_state.borrow_mut();

        // TODO figure out a better way to do this - we don't have an easy
        // way for the targeter to cause a layout of the label
        if let Some(targeter) = state.targeter() {
            let mut targeter_label = self.targeter_label.borrow_mut();
            if !targeter_label.state.is_visible() {
                targeter_label.state.set_visible(true);
                // temporarily clear text so we don't show old data on this frame
                targeter_label.state.text = String::new();
                targeter_label.state.add_text_arg("ability_id", targeter.borrow().name());
                targeter_label.invalidate_layout();
            }
        } else {
            self.targeter_label.borrow_mut().state.set_visible(false);
        }

        match state.pop_scroll_to_callback() {
            None => (),
            Some(entity) => self.center_scroll_on(&entity, state.area.width, state.area.height, widget),
        }

        if self.cache_invalid {
            debug!("Caching area '{}' layers to texture", state.area.id);

            let texture_ids = vec![VISIBILITY_TEX_ID, BASE_LAYER_ID, AERIAL_LAYER_ID, ENTITY_TEX_ID];
            for texture_id in texture_ids {
                if renderer.has_texture(texture_id) {
                    renderer.clear_texture(texture_id);
                } else {
                    renderer.register_texture(texture_id,
                                              ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE, TILE_CACHE_TEXTURE_SIZE),
                                              TextureMinFilter::NearestMipmapNearest,
                                              TextureMagFilter::Nearest);
                }
            }

            for (index, layer) in state.area.layer_set.layers.iter().enumerate() {
                let texture_id = if index <= state.area.layer_set.entity_layer_index {
                    BASE_LAYER_ID
                } else {
                    AERIAL_LAYER_ID
                };
                trace!("Caching layer '{}'", layer.id);

                self.draw_layer_to_texture(renderer, &layer, texture_id);
            }

            let (max_x, max_y) = AreaView::get_texture_cache_max(state.area.width, state.area.height);
            trace!("Full area visibility draw from 0,0 to {},{}", max_x, max_y);
            self.draw_vis_to_texture(renderer, &state.area.visibility_tile, &state.area.explored_tile,
                                     &state, 0, 0, max_x, max_y);
            self.cache_invalid = false;
            state.pc_vis_delta = (false, 0, 0);
        }

        let (redraw, pc_vis_delta_x, pc_vis_delta_y) = state.pc_vis_delta;
        if redraw {
            state.pc_vis_delta = (false, 0, 0);
            trace!("Redrawing PC visibility to texture");
            self.draw_visibility_to_texture(renderer, &state.area.visibility_tile,
                                            &state.area.explored_tile, &state,
                                            pc_vis_delta_x, pc_vis_delta_y);
        }

        let p = widget.state.inner_position;

        self.draw_layer(renderer, scale_x, scale_y, widget, BASE_LAYER_ID);
        GameState::draw_below_entities(renderer, p.x as f32 - self.scroll.x(), p.y as f32- self.scroll.y(),
            scale_x, scale_y, millis);

        let mut draw_list = DrawList::empty_sprite();
        for transition in state.area.transitions.iter() {
            draw_list.set_scale(scale_x, scale_y);
            transition.image_display.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                                        (transition.from.x + p.x) as f32 - self.scroll.x(),
                                                        (transition.from.y + p.y) as f32 - self.scroll.y(),
                                                        transition.size.width as f32,
                                                        transition.size.height as f32,
                                                        millis);
        }

        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        for selected in GameState::selected() {
            self.draw_selection(&selected, renderer, scale_x, scale_y, widget, millis);
        }
        for entity in self.select_party_in_box(widget).iter() {
            self.draw_selection(&entity, renderer, scale_x, scale_y, widget, millis);
        }

        self.draw_entities_props(renderer, scale_x, scale_y, 1.0, widget, &state, millis);
        GameState::draw_above_entities(renderer, p.x as f32 - self.scroll.x(), p.y as f32- self.scroll.y(),
            scale_x, scale_y, millis);
        self.draw_layer(renderer, scale_x, scale_y, widget, AERIAL_LAYER_ID);

        if let Some(ref hover) = self.hover_sprite {
            let mut draw_list = DrawList::from_sprite_f32(&hover.sprite,
                                                          (hover.x + p.x) as f32 - self.scroll.x(),
                                                          (hover.y + p.y) as f32 - self.scroll.y(),
                                                          hover.w as f32, hover.h as f32);
            if !hover.left_click_action_valid { draw_list.set_color(color::RED); }
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        let targeter_tile = match self.targeter_tile {
            None => return,
            Some(ref tile) => Rc::clone(tile),
        };

        if let Some(ref targeter) = state.targeter() {
            targeter.borrow().draw(renderer, &targeter_tile, self.scroll.x(), self.scroll.y(),
                scale_x, scale_y, millis);
        }

        self.draw_entities_props(renderer, scale_x, scale_y, 0.6, widget, &state, millis);
        self.draw_layer(renderer, scale_x, scale_y, widget, VISIBILITY_TEX_ID);

        if let Some(ref image) = self.selection_box_image {
            if let Some((x, y, x_end, y_end)) = self.get_selection_box_coords() {
                let w = x_end - x;
                let h = y_end - y;

                if w < 1.0 || h < 1.0 { return; }

                let mut draw_list = DrawList::empty_sprite();
                image.append_to_draw_list(&mut draw_list, &animation_state::NORMAL, x, y, w, h, millis);
                renderer.draw(draw_list);
            }
        }

        for feedback_text in state.feedback_text_iter() {
            feedback_text.draw(renderer, &self.feedback_text_params,
                               p.x as f32 - self.scroll.x(), p.y as f32- self.scroll.y(),
                               scale_x, scale_y);
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let (x, y) = self.get_cursor_pos(widget);
        if x < 0 || y < 0 { return true; }

        if kind == ClickKind::Middle { return true; }

        let area_state = GameState::area_state();

        let targeter = area_state.borrow_mut().targeter();
        if let Some(targeter) = targeter {
            match kind {
                ClickKind::Left => targeter.borrow_mut().on_activate(),
                ClickKind::Right => targeter.borrow_mut().on_cancel(),
                _ => (),
            }
        } else {
            let mut fire_action = false;
            match kind {
                ClickKind::Left => {
                    if let Some((x, y, x_end, y_end)) = self.get_selection_box_coords() {
                        let w = x_end - x;
                        let h = y_end - y;
                        if w < 1.0 && h < 1.0 { fire_action = true; }
                        else {
                            GameState::select_party_members(self.select_party_in_box(&widget.borrow()));
                        }
                        self.selection_box_start = None;
                    } else {
                        fire_action = true;
                    }
                },
                _ => (),
            }

            if fire_action {
                let mut action = action_kind::get_action(x, y);
                action.fire_action(widget);
            }
        }
        true
    }

    fn on_mouse_drag(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind,
                     delta_x: f32, delta_y: f32) -> bool {
        let area_state = GameState::area_state();
        let area_width = area_state.borrow().area.width;
        let area_height = area_state.borrow().area.height;
        self.scroll.compute_max(&*widget.borrow(), area_width, area_height,
            self.scale.0, self.scale.1);

        match kind {
            ClickKind::Middle => {
                self.scroll(delta_x, delta_y, 33);
            },
            ClickKind::Left => {
                if let Some(_) = area_state.borrow_mut().targeter() {
                    return true;
                }

                match self.selection_box_start {
                    None => {
                        self.selection_box_start = Some(Cursor::get_position_f32());
                    }, Some(_) => (),
                }
            },
            _ => (),
        }

        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.hover_sprite = None;
        self.selection_box_start = None;
        true
    }
}
