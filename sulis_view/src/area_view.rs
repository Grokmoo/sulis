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
use std::cell::{RefCell, RefMut};
use std::cmp::{self, Ordering};
use std::mem;
use std::rc::Rc;
use std::time;

use sulis_core::config::Config;
use sulis_core::extern_image::ImageBuffer;
use sulis_core::image::Image;
use sulis_core::io::event::ClickKind;
use sulis_core::io::*;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::ui::{animation_state, compute_area_scaling};
use sulis_core::ui::{color, Color, Cursor, Scrollable, Widget, WidgetKind};
use sulis_core::util::{self, Offset, Point, Rect, Scale};
use sulis_core::widgets::Label;
use sulis_module::{
    area::{Layer, Tile},
    DamageKind, Module,
};
use sulis_state::{area_feedback_text, area_state::PCVisRedraw, RangeIndicatorImageSet};
use sulis_state::{AreaDrawable, AreaState, EntityState, EntityTextureCache, GameState};

use crate::{action_kind, window_fade, AreaOverlayHandler, ScreenShake, WindowFade};

struct Range {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

const NAME: &str = "area";

pub struct AreaView {
    scale: (f32, f32),
    cache_invalid: bool,
    layers: Vec<String>,
    entity_texture_cache: EntityTextureCache,

    targeter_label: Rc<RefCell<Widget>>,
    targeter_tile: Option<Rc<dyn Image>>,
    range_indicator_image_set: Option<RangeIndicatorImageSet>,

    scroll: Scrollable,
    active_entity: Option<Rc<RefCell<EntityState>>>,
    feedback_text_params: area_feedback_text::Params,
    entity_see_through_alpha: f32,

    scroll_target: Option<(f32, f32)>,
    screen_shake: Option<ScreenShake>,

    overlay_handler: AreaOverlayHandler,
}

const TILE_CACHE_TEXTURE_SIZE: u32 = 2048;
const TILE_SIZE: u32 = 16;
const TEX_COORDS: [f32; 8] = [0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];

const ENTITY_TEX_ID: &str = "__entities__";
const VISIBILITY_TEX_ID: &str = "__visibility__";
const BASE_LAYER_ID: &str = "__base_layer__";
const AERIAL_LAYER_ID: &str = "__aerial_layer__";

impl AreaView {
    pub fn new() -> Rc<RefCell<AreaView>> {
        Rc::new(RefCell::new(AreaView {
            targeter_label: Widget::with_theme(Label::empty(), "targeter_label"),
            scale: (1.0, 1.0),
            cache_invalid: true,
            entity_texture_cache: EntityTextureCache::new(
                ENTITY_TEX_ID,
                TILE_CACHE_TEXTURE_SIZE,
                TILE_SIZE,
            ),
            layers: Vec::new(),
            scroll: Scrollable::default(),
            targeter_tile: None,
            range_indicator_image_set: None,
            active_entity: None,
            entity_see_through_alpha: 0.2,
            feedback_text_params: area_feedback_text::Params::default(),
            scroll_target: None,
            screen_shake: None,
            overlay_handler: AreaOverlayHandler::default(),
        }))
    }

    pub fn clear_mouse_state(&mut self) {
        self.overlay_handler.clear_mouse_state();
    }

    pub fn clear_area_mouseover(&mut self) {
        self.overlay_handler.clear_area_mouseover();
    }

    pub fn update_cursor_and_hover(&mut self, widget: &Rc<RefCell<Widget>>) {
        let (x, y) = self.get_cursor_pos(widget);
        self.overlay_handler.update_cursor_and_hover(widget, x, y);
    }

    pub fn center_scroll_on(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        area_width: i32,
        area_height: i32,
        widget: &Widget,
    ) {
        let x = entity.borrow().location.x as f32 + entity.borrow().size.width as f32 / 2.0;
        let y = entity.borrow().location.y as f32 + entity.borrow().size.height as f32 / 2.0;

        let (x, y) = self.center_scroll_on_point(x, y, area_width, area_height, widget);
        self.scroll.set(x, y);
    }

    fn center_scroll_on_point(
        &mut self,
        x: f32,
        y: f32,
        area_width: i32,
        area_height: i32,
        widget: &Widget,
    ) -> (f32, f32) {
        let (scale_x, scale_y) = self.scale;
        self.scroll
            .compute_max(widget, area_width, area_height, scale_x, scale_y);

        let x = x - widget.state.inner_width() as f32 / scale_x / 2.0;
        let y = y - widget.state.inner_height() as f32 / scale_y / 2.0;

        (x, y)
    }

    pub fn screen_shake(&mut self) {
        if !Config::crit_screen_shake() { return; }

        self.screen_shake = Some(ScreenShake::new());
    }

    pub fn delayed_scroll_to_point(
        &mut self,
        x: f32,
        y: f32,
        area_width: i32,
        area_height: i32,
        widget: &Widget,
    ) {
        let (x, y) = self.center_scroll_on_point(x, y, area_width, area_height, widget);
        let (x, y) = self.scroll.bound(x, y);
        self.scroll_target = Some((x, y));
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>) -> (f32, f32) {
        let pos = widget.borrow().state.inner_position();
        let (x, y) = self.get_cursor_pos_scaled(pos.x, pos.y);
        ((x + self.scroll.x()), (y + self.scroll.y()))
    }

    fn get_cursor_pos_scaled(&self, pos_x: i32, pos_y: i32) -> (f32, f32) {
        let mut x = Cursor::get_x_f32() - pos_x as f32;
        let mut y = Cursor::get_y_f32() - pos_y as f32;

        let (scale_x, scale_y) = self.scale;
        x /= scale_x;
        y /= scale_y;

        (x, y)
    }

    fn draw_layer_to_texture(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        layer: &Layer,
        texture_id: &str,
    ) {
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
        tiles.sort_by(|a, b| (a.1 + a.2.height).cmp(&(b.1 + b.2.height)));

        let mut draw_list = DrawList::empty_sprite();
        for (x, y, tile) in tiles {
            let rect = Rect {
                x: x as f32,
                y: y as f32,
                w: tile.width as f32,
                h: tile.height as f32,
            };
            draw_list.append(&mut DrawList::from_sprite(&tile.image_display, rect));
        }

        if !draw_list.is_empty() {
            AreaView::draw_list_to_texture(renderer, draw_list, texture_id);
        }
    }

    fn draw_visibility_to_texture(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        vis_sprite: &Rc<Sprite>,
        explored_sprite: &Rc<Sprite>,
        area_state: &RefMut<AreaState>,
        delta_x: i32,
        delta_y: i32,
    ) {
        let start_time = time::Instant::now();
        let (max_tile_x, max_tile_y) =
            AreaView::get_texture_cache_max(area_state.area.width, area_state.area.height);

        let vis_dist = area_state.area.area.vis_dist;
        for pc in GameState::party() {
            let c_x = pc.borrow().location.x + pc.borrow().size.width / 2;
            let c_y = pc.borrow().location.y + pc.borrow().size.height / 2;
            let min_x = cmp::max(0, c_x - vis_dist + if delta_x < 0 { delta_x } else { 0 });
            let max_x = cmp::min(
                max_tile_x,
                1 + c_x + vis_dist + if delta_x > 0 { delta_x } else { 0 },
            );
            let min_y = cmp::max(0, c_y - vis_dist + if delta_y < 0 { delta_y } else { 0 });
            let max_y = cmp::min(
                max_tile_y,
                1 + c_y + vis_dist + if delta_y > 0 { delta_y } else { 0 },
            );

            let scale = TILE_SIZE as i32;
            renderer.clear_texture_region(
                VISIBILITY_TEX_ID,
                min_x * scale,
                min_y * scale,
                max_x * scale,
                max_y * scale,
            );
            let range = Range {
                min_x,
                max_x,
                min_y,
                max_y,
            };
            self.draw_vis_to_texture(renderer, vis_sprite, explored_sprite, area_state, range);
            trace!(
                "Visibility render to texture time: {}",
                util::format_elapsed_secs(start_time.elapsed())
            );
        }
    }

    fn draw_vis_to_texture(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        vis_sprite: &Rc<Sprite>,
        explored_sprite: &Rc<Sprite>,
        area_state: &RefMut<AreaState>,
        range: Range,
    ) {
        let mut draw_list = DrawList::empty_sprite();

        // info!("======");
        for tile_y in range.min_y..range.max_y {
            // let mut cur_line = "".to_string();
            for tile_x in range.min_x..range.max_x {
                if area_state.is_pc_visible(tile_x, tile_y) {
                    // cur_line.push('x');
                    continue;
                } else {
                    // cur_line.push(' ');
                }

                let rect = Rect {
                    x: tile_x as f32,
                    y: tile_y as f32,
                    w: 1.0,
                    h: 1.0,
                };
                draw_list.append(&mut DrawList::from_sprite(vis_sprite, rect));

                if area_state.is_pc_explored(tile_x, tile_y) {
                    continue;
                }
                draw_list.append(&mut DrawList::from_sprite(explored_sprite, rect));
            }
            // info!("{}|", cur_line);
        }

        if draw_list.is_empty() {
            return;
        }
        AreaView::draw_list_to_texture(renderer, draw_list, VISIBILITY_TEX_ID);
    }

    fn draw_list_to_texture(
        renderer: &mut dyn GraphicsRenderer,
        draw_list: DrawList,
        texture_id: &str,
    ) {
        let (ui_x, ui_y) = Config::ui_size();
        let mut draw_list = draw_list;
        draw_list.texture_mag_filter = TextureMagFilter::Linear;
        draw_list.texture_min_filter = TextureMinFilter::Linear;
        draw_list.set_scale(Scale {
            x: TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 * ui_x as f32,
            y: TILE_SIZE as f32 / TILE_CACHE_TEXTURE_SIZE as f32 * ui_y as f32,
        });
        renderer.draw_to_texture(texture_id, draw_list);
    }

    fn get_texture_cache_max(width: i32, height: i32) -> (i32, i32) {
        let x = cmp::min((TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as i32, width);
        let y = cmp::min((TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as i32, height);

        (x, y)
    }

    fn draw_layer(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        scale: Scale,
        widget: &Widget,
        id: &str,
        color: Color,
    ) {
        let p = widget.state.inner_position();
        let rect = Rect {
            x: p.x as f32 - self.scroll.x(),
            y: p.y as f32 - self.scroll.y(),
            w: (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32,
            h: (TILE_CACHE_TEXTURE_SIZE / TILE_SIZE) as f32,
        };
        let mut draw_list = DrawList::from_texture_id(&id, &TEX_COORDS, rect);
        draw_list.set_scale(scale);
        draw_list.set_color(color);
        renderer.draw(draw_list);
    }

    fn draw_entities_props(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        scale: Scale,
        color: Color,
        widget: &Widget,
        state: &AreaState,
        millis: u32,
    ) {
        // let start_time = time::Instant::now();
        let mut to_draw: Vec<&dyn AreaDrawable> = Vec::new();

        for prop_state in state.props().iter() {
            to_draw.push(&*prop_state);
        }

        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        for index in state.entity_iter() {
            let entity = mgr.entity(*index);
            let mut entity = entity.borrow_mut();
            if !entity
                .location_points()
                .any(|p| state.is_pc_visible(p.x, p.y))
            {
                continue;
            }

            entity.cache(renderer, &mut self.entity_texture_cache);

            let entity = unsafe { mem::transmute::<&EntityState, &'static EntityState>(&*entity) };

            to_draw.push(entity);
        }

        to_draw.sort_by(|a, b| {
            if a.aerial() && !b.aerial() {
                std::cmp::Ordering::Greater
            } else if !a.aerial() && b.aerial() {
                std::cmp::Ordering::Less
            } else {
                let a_y = a.location().y + a.size().height;
                let a_x = a.location().x + a.size().width / 2;
                let b_y = b.location().y + b.size().height;
                let b_x = b.location().x + b.size().width / 2;

                if a_y < b_y { Ordering::Less }
                else if a_y > b_y { Ordering::Greater }
                else if a_x < b_x { Ordering::Less }
                else if a_x > b_x { Ordering::Greater }
                else { Ordering::Equal }
            }
        });

        for drawable in to_draw {
            let (x, y) = widget.state.inner_position().as_tuple();
            let (x, y) = (x as f32 - self.scroll.x(), y as f32 - self.scroll.y());
            drawable.draw(renderer, scale, x, y, millis, color);
        }

        // info!("Entity & Prop draw time: {}", util::format_elapsed_secs(start_time.elapsed()));
    }

    fn draw_selection(
        &mut self,
        selected: &Rc<RefCell<EntityState>>,
        renderer: &mut dyn GraphicsRenderer,
        scale: Scale,
        widget: &Widget,
        millis: u32,
    ) {
        let x_base = widget.state.inner_left() as f32 - self.scroll.x();
        let y_base = widget.state.inner_top() as f32 - self.scroll.y();

        let selected = selected.borrow();
        let w = selected.size.width as f32;
        let h = selected.size.height as f32;
        let x = x_base + selected.location.x as f32 + selected.sub_pos.0;
        let y = y_base + selected.location.y as f32 + selected.sub_pos.1;

        let rect = Rect { x, y, w, h };
        let mut draw_list = DrawList::empty_sprite();
        selected.size.selection_image.append_to_draw_list(
            &mut draw_list,
            &animation_state::NORMAL,
            rect,
            millis,
        );

        draw_list.set_scale(scale);
        renderer.draw(draw_list);
    }

    pub fn scroll(&mut self, delta_x: f32, delta_y: f32, millis: u32) {
        let speed = Config::scroll_speed() * millis as f32 / 33.0;
        let delta_x = speed * delta_x / self.scale.0;
        let delta_y = speed * delta_y / self.scale.1;
        self.scroll.change(delta_x, delta_y)
    }

    pub fn set_active_entity(&mut self, entity: Option<Rc<RefCell<EntityState>>>) {
        self.active_entity = entity;
    }

    fn handle_targeter_label(&mut self, state: &mut AreaState) {
        if let Some(targeter) = state.targeter() {
            let mut targeter_label = self.targeter_label.borrow_mut();
            if !targeter_label.state.is_visible() {
                targeter_label.state.set_visible(true);
                // temporarily clear text so we don't show old data on this frame
                targeter_label.state.text = String::new();
                targeter_label
                    .state
                    .add_text_arg("ability_id", targeter.borrow().name());
                targeter_label.invalidate_layout();
            }
        } else {
            self.targeter_label.borrow_mut().state.set_visible(false);
        }
    }

    fn cache_textures(&mut self, renderer: &mut dyn GraphicsRenderer, state: &mut AreaState) {
        info!("Caching area '{}' layers to texture", state.area.area.id);

        let texture_ids = vec![
            VISIBILITY_TEX_ID,
            BASE_LAYER_ID,
            AERIAL_LAYER_ID,
            ENTITY_TEX_ID,
        ];
        for texture_id in texture_ids {
            if renderer.has_texture(texture_id) {
                renderer.clear_texture(texture_id);
            } else {
                renderer.register_texture(
                    texture_id,
                    ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE, TILE_CACHE_TEXTURE_SIZE),
                    TextureMinFilter::NearestMipmapNearest,
                    TextureMagFilter::Nearest,
                );
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

        self.entity_texture_cache.invalidate();
        // cause full area visibility redraw at the next step
        state.pc_vis_full_redraw();
        self.cache_invalid = false;
    }
}

impl WidgetKind for AreaView {
    widget_kind!(NAME);

    #[allow(clippy::float_cmp)]
    fn update(&mut self, _widget: &Rc<RefCell<Widget>>, millis: u32) {
        if let Some(shake) = self.screen_shake.as_mut() {
            let result = shake.shake(millis);

            if let Some(scroll) = result.scroll {
                let factor = GameState::user_zoom();
                self.scroll.change(scroll.x / factor, scroll.y / factor);
            }

            if result.done {
                self.screen_shake = None;
            }
        }

        let (dest_x, dest_y) = match self.scroll_target {
            None => return,
            Some((x, y)) => (x, y),
        };

        let (sign_x, sign_y) = (
            (self.scroll.x() - dest_x).signum(),
            (self.scroll.y() - dest_y).signum(),
        );

        let speed = Config::scroll_speed() * 4.0 * millis as f32 / 33.3;
        let (speed_x, speed_y) = (speed / self.scale.0, speed / self.scale.1);

        let (cur_x, cur_y) = (self.scroll.x(), self.scroll.y());

        let (dir_x, dir_y) = (cur_x - dest_x, cur_y - dest_y);

        let mag = dir_y.hypot(dir_x);
        let (delta_x, delta_y) = (dir_x * speed_x / mag, dir_y * speed_y / mag);

        self.scroll.change(delta_x, delta_y);

        if (self.scroll.x() - dest_x).signum() != sign_x
            || (self.scroll.y() - dest_y).signum() != sign_y
        {
            self.scroll.set(dest_x, dest_y);
            self.scroll_target = None;
        }
    }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        let theme = &widget.theme;
        let selection_prefix = theme
            .custom
            .get("selection_image_prefix")
            .map(|s| s.as_ref())
            .unwrap_or("selection_area_");

        let image_set = RangeIndicatorImageSet::new(selection_prefix.to_string());
        self.range_indicator_image_set = Some(image_set);

        self.overlay_handler.apply_theme(theme);

        if let Some(ref image_id) = theme.custom.get("targeter_tile") {
            self.targeter_tile = ResourceSet::image(image_id);
        }

        self.entity_see_through_alpha = theme.get_custom_or_default("entity_see_through_alpha", 0.2);
        self.feedback_text_params.scale = theme.get_custom_or_default("feedback_text_scale", 1.0);
        self.feedback_text_params.ap_scale =
            theme.get_custom_or_default("ap_hover_text_scale", 1.0);
        self.feedback_text_params.ap_color =
            theme.get_custom_or_default("ap_hover_text_color", color::LIGHT_GRAY);

        if let Some(ref font_id) = theme.custom.get("feedback_text_font") {
            self.feedback_text_params.font = match ResourceSet::font(font_id) {
                None => {
                    warn!(
                        "Invalid font specified for area feedback text '{}'",
                        font_id
                    );
                    ResourceSet::default_font()
                }
                Some(font) => font,
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

        for kind in DamageKind::iter() {
            let id = format!(
                "feedback_text_damage_{}_color",
                kind.to_str().to_lowercase()
            );
            let index = kind.index();
            self.feedback_text_params.damage_colors[index] =
                theme.get_custom_or_default(&id, color::LIGHT_GRAY);
        }

        if let Some(ref image_id) = theme.custom.get("feedback_icon_concealment") {
            self.feedback_text_params.concealment_icon = ResourceSet::image_else_empty(image_id);
        }

        if let Some(ref image_id) = theme.custom.get("feedback_icon_backstab") {
            self.feedback_text_params.backstab_icon = ResourceSet::image_else_empty(image_id);
        }
        if let Some(ref image_id) = theme.custom.get("feedback_icon_flanking") {
            self.feedback_text_params.flanking_icon = ResourceSet::image_else_empty(image_id);
        }

        if let Some(ref image_id) = theme.custom.get("feedback_icon_crit") {
            self.feedback_text_params.crit_icon = ResourceSet::image_else_empty(image_id);
        }
        if let Some(ref image_id) = theme.custom.get("feedback_icon_hit") {
            self.feedback_text_params.hit_icon = ResourceSet::image_else_empty(image_id);
        }
        if let Some(ref image_id) = theme.custom.get("feedback_icon_graze") {
            self.feedback_text_params.graze_icon = ResourceSet::image_else_empty(image_id);
        }

        if self.targeter_tile.is_none() {
            warn!("No targeter tile specified for Areaview");
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        info!("Adding area to widget tree");
        self.overlay_handler = AreaOverlayHandler::default();

        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;

        for layer in area.layer_set.layers.iter() {
            self.layers.push(layer.id.to_string());
        }
        self.cache_invalid = true;

        let fade = Widget::with_defaults(WindowFade::new(window_fade::Mode::In));

        vec![Rc::clone(&self.targeter_label), fade]
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
        self.scale = (
            old_scale_x / old_user_scale * user_scale,
            old_scale_y / old_user_scale * user_scale,
        );

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

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
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
        self.handle_targeter_label(&mut state);

        if let Some(entity) = state.pop_scroll_to_callback() {
            self.center_scroll_on(&entity, state.area.width, state.area.height, widget)
        }

        if self.cache_invalid {
            self.cache_textures(renderer, &mut state);
        }

        match state.take_pc_vis() {
            PCVisRedraw::Full => {
                let (max_x, max_y) =
                    AreaView::get_texture_cache_max(state.area.width, state.area.height);
                trace!("Full area visibility draw from 0,0 to {},{}", max_x, max_y);
                let range = Range {
                    min_x: 0,
                    max_x,
                    min_y: 0,
                    max_y,
                };
                self.draw_vis_to_texture(
                    renderer,
                    &state.area.area.visibility_tile,
                    &state.area.area.explored_tile,
                    &state,
                    range,
                );
            }
            PCVisRedraw::Partial { delta_x, delta_y } => {
                trace!("Redrawing PC visibility to texture");
                self.draw_visibility_to_texture(
                    renderer,
                    &state.area.area.visibility_tile,
                    &state.area.area.explored_tile,
                    &state,
                    delta_x,
                    delta_y,
                );
            }
            PCVisRedraw::Not => (),
        }

        let p = widget.state.inner_position();

        let rules = Module::rules();
        let mgr = GameState::turn_manager();
        let time = mgr.borrow().current_time();
        let area_color = rules.get_area_color(state.area.area.location_kind, time);

        let scale = Scale {
            x: scale_x,
            y: scale_y,
        };
        self.draw_layer(renderer, scale, widget, BASE_LAYER_ID, area_color);
        GameState::draw_below_entities(
            renderer,
            Offset {
                x: p.x as f32 - self.scroll.x(),
                y: p.y as f32 - self.scroll.y(),
            },
            scale,
            millis,
        );

        let image_set = match self.range_indicator_image_set {
            None => return,
            Some(ref set) => set,
        };

        let offset = Offset {
            x: self.scroll.x(),
            y: self.scroll.y(),
        };
        if let Some(ref indicator) = state.range_indicator() {
            let mut draw_list = indicator.get_draw_list(image_set, offset, millis);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        if let Some(mut draw_list) = self.overlay_handler.get_path_draw_list(offset, millis) {
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        let mut draw_list = DrawList::empty_sprite();
        for transition in state.area.transitions.iter() {
            draw_list.set_scale(scale);
            let rect = Rect {
                x: (transition.from.x + p.x) as f32 - self.scroll.x(),
                y: (transition.from.y + p.y) as f32 - self.scroll.y(),
                w: transition.size.width as f32,
                h: transition.size.height as f32,
            };
            transition.image_display.append_to_draw_list(
                &mut draw_list,
                &animation_state::NORMAL,
                rect,
                millis,
            );
        }

        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        let active_entity = self.active_entity.clone();
        if let Some(ref entity) = active_entity {
            self.draw_selection(entity, renderer, scale, widget, millis);
        } else {
            for selected in GameState::selected() {
                self.draw_selection(&selected, renderer, scale, widget, millis);
            }

            let scroll = (self.scroll.x(), self.scroll.y());
            let party = self
                .overlay_handler
                .select_party_in_box(widget, self.scale, scroll);
            for entity in party.iter() {
                self.draw_selection(&entity, renderer, scale, widget, millis);
            }
        }

        self.draw_entities_props(renderer, scale, area_color, widget, &state, millis);
        let offset = Offset {
            x: p.x as f32 - self.scroll.x(),
            y: p.y as f32 - self.scroll.y(),
        };
        GameState::draw_above_entities(renderer, offset, scale, millis);
        self.draw_layer(renderer, scale, widget, AERIAL_LAYER_ID, area_color);

        if let Some(ref hover) = self.overlay_handler.hover_sprite() {
            let rect = Rect {
                x: (hover.x + p.x) as f32 - self.scroll.x(),
                y: (hover.y + p.y) as f32 - self.scroll.y(),
                w: hover.w as f32,
                h: hover.h as f32,
            };

            let mut draw_list = DrawList::from_sprite_f32(&hover.sprite, rect);
            if !hover.left_click_action_valid {
                draw_list.set_color(color::RED);
            }
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        let targeter_tile = match self.targeter_tile {
            None => return,
            Some(ref tile) => Rc::clone(tile),
        };

        if let Some(ref targeter) = state.targeter() {
            let offset = Offset {
                x: self.scroll.x(),
                y: self.scroll.y(),
            };
            targeter.borrow_mut().draw(
                renderer,
                &targeter_tile,
                offset,
                scale,
                millis,
                &self.feedback_text_params,
            );
        }

        let color = Color::new(area_color.r, area_color.g, area_color.b,
            self.entity_see_through_alpha * area_color.a);
        self.draw_entities_props(renderer, scale, color, widget, &state, millis);

        if Config::debug().limit_line_of_sight {
            self.draw_layer(renderer, scale, widget, VISIBILITY_TEX_ID, color::WHITE);
        }

        let offset = Offset {
            x: p.x as f32 - self.scroll.x(),
            y: p.y as f32 - self.scroll.y(),
        };
        self.overlay_handler
            .draw_top(renderer, &self.feedback_text_params, offset, scale, millis);

        for feedback_text in state.feedback_text_iter_mut() {
            feedback_text.draw(renderer, &self.feedback_text_params, offset, scale, millis);
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let (x, y) = self.get_cursor_pos(widget);
        if x < 0.0 || y < 0.0 {
            return true;
        }

        if kind == ClickKind::Tertiary {
            return true;
        }

        let area_state = GameState::area_state();

        let targeter = area_state.borrow_mut().targeter();
        if let Some(targeter) = targeter {
            match kind {
                ClickKind::Primary => targeter.borrow_mut().on_activate(),
                ClickKind::Secondary => targeter.borrow_mut().on_cancel(),
                _ => (),
            }
        } else {
            let scroll = (self.scroll.x(), self.scroll.y());
            let fire_action = match kind {
                ClickKind::Primary => self
                    .overlay_handler
                    .handle_left_click(widget, self.scale, scroll),
                _ => false,
            };

            if fire_action {
                let mut action = action_kind::get_action(x, y);
                let clear_mouse_state = action.fire_action(widget);

                if clear_mouse_state {
                    self.overlay_handler.clear_mouse_state();
                }
            }
        }
        true
    }

    fn on_mouse_drag(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        kind: ClickKind,
        delta_x: f32,
        delta_y: f32,
    ) -> bool {
        let area_state = GameState::area_state();
        let area_width = area_state.borrow().area.width;
        let area_height = area_state.borrow().area.height;
        self.scroll.compute_max(
            &*widget.borrow(),
            area_width,
            area_height,
            self.scale.0,
            self.scale.1,
        );

        match kind {
            ClickKind::Tertiary => {
                self.scroll(delta_x, delta_y, 33);
            }
            ClickKind::Primary => {
                if area_state.borrow_mut().targeter().is_some() {
                    return true;
                }

                self.overlay_handler.handle_left_drag();
            }
            _ => (),
        }

        true
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.set_active_entity(None);
        self.super_on_mouse_enter(widget);
        true
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.overlay_handler.on_mouse_exit();
        true
    }
}
