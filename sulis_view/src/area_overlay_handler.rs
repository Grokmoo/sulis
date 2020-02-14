//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::cell::RefCell;
use std::rc::Rc;

use crate::{action_kind, AreaMouseover};
use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::ui::{animation_state, Cursor, LineRenderer, Theme, Widget};
use sulis_module::Module;
use sulis_state::{area_feedback_text::Params, AreaState, EntityState, GameState};

pub struct HoverSprite {
    pub sprite: Rc<Sprite>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub left_click_action_valid: bool,
}

pub struct AreaOverlayHandler {
    hover_sprite: Option<HoverSprite>,
    selection_box_start: Option<(f32, f32)>,
    selection_box_image: Option<Rc<dyn Image>>,

    area_mouseover: Option<Rc<RefCell<AreaMouseover>>>,
    area_mouseover_widget: Option<Rc<RefCell<Widget>>>,

    path: Vec<(f32, f32)>,
    path_point_image: Option<Rc<dyn Image>>,
    path_point_end_image: Option<Rc<dyn Image>>,
    path_ap: Option<i32>,
}

impl Default for AreaOverlayHandler {
    fn default() -> AreaOverlayHandler {
        AreaOverlayHandler {
            hover_sprite: None,
            selection_box_start: None,
            area_mouseover: None,
            area_mouseover_widget: None,
            selection_box_image: None,
            path: Vec::new(),
            path_point_image: None,
            path_point_end_image: None,
            path_ap: None,
        }
    }
}

impl AreaOverlayHandler {
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

    pub fn select_party_in_box(
        &self,
        widget: &Widget,
        scale: (f32, f32),
        scroll: (f32, f32),
    ) -> Vec<Rc<RefCell<EntityState>>> {
        let mut party = Vec::new();

        let (x, y, x_end, y_end) = match self.get_selection_box_coords() {
            None => return party,
            Some((x, y, x2, y2)) => (x, y, x2, y2),
        };

        let pos = widget.state.inner_position();
        let x1 = ((x - pos.x as f32) / scale.0 + scroll.0) as i32;
        let y1 = ((y - pos.y as f32) / scale.1 + scroll.1) as i32;
        let x2 = ((x_end - pos.x as f32) / scale.0 + scroll.0) as i32;
        let y2 = ((y_end - pos.y as f32) / scale.1 + scroll.1) as i32;

        for entity in GameState::party().iter() {
            let loc = &entity.borrow().location;
            let size = &entity.borrow().size;

            if loc.x >= x2 || x1 >= loc.x + size.width || loc.y >= y2 || y1 >= loc.y + size.height {
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
            let pc = GameState::player();
            if pc.borrow().is_hostile(&entity.borrow()) && entity.borrow().actor.stats.hidden {
                None
            } else {
                Some(AreaMouseover::new_entity(&entity))
            }
        } else if self.check_closed_door(x, y, &area_state).is_some() {
            None
        } else if let Some(transition) = area_state.get_transition_at(x, y) {
            Some(AreaMouseover::new_transition(&transition.hover_text))
        } else if let Some(index) = area_state.props().index_at(x, y) {
            let interactive = {
                let prop = area_state.props().get(index);
                (prop.is_container() || prop.is_hover()) && prop.is_enabled()
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

    fn check_closed_door(
        &self,
        x: i32,
        y: i32,
        area_state: &AreaState,
    ) -> Option<Rc<RefCell<AreaMouseover>>> {
        if let Some(index) = area_state.props().index_at(x, y) {
            {
                let prop = area_state.props().get(index);
                if !prop.is_door() || prop.is_active() {
                    return None;
                }
            }

            Some(AreaMouseover::new_prop(index))
        } else {
            None
        }
    }

    fn set_cursor(&mut self, x: f32, y: f32) {
        let action = action_kind::get_action(x, y);
        Cursor::set_cursor_state(action.cursor_state());
        match action.get_hover_info() {
            Some(info) => {
                let hover_sprite = HoverSprite {
                    sprite: Rc::clone(&info.size.cursor_sprite),
                    x: info.x,
                    y: info.y,
                    w: info.size.width,
                    h: info.size.height,
                    left_click_action_valid: true,
                };
                self.hover_sprite = Some(hover_sprite);
                self.path = info.path;
                match info.ap {
                    0 => self.path_ap = None,
                    ap => self.path_ap = Some(info.total_ap - ap),
                }
            }
            None => {
                self.hover_sprite = None;
                self.path.clear();
                self.path_ap = None;
            }
        }
    }

    pub fn update_cursor_and_hover(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        area_x: f32,
        area_y: f32,
    ) {
        let area_state = GameState::area_state();

        self.hover_sprite = None;

        if self.selection_box_start.is_some() {
            Cursor::set_cursor_state(animation_state::Kind::Normal);
            return;
        }

        match self.check_mouseover(area_x as i32, area_y as i32) {
            None => {
                self.clear_area_mouseover();
            }
            Some(mouseover) => {
                let (clear, set_new) = if let Some(ref cur_mouseover) = self.area_mouseover {
                    if *cur_mouseover.borrow() == *mouseover.borrow() {
                        (false, false)
                    } else {
                        (true, true)
                    }
                } else {
                    (false, true)
                };

                if clear {
                    self.clear_area_mouseover();
                }
                if set_new {
                    self.set_new_mouseover(widget, mouseover);
                }
            }
        };

        if area_state.borrow_mut().targeter().is_none() {
            self.set_cursor(area_x, area_y);
        }
    }

    fn set_new_mouseover(
        &mut self,
        parent: &Rc<RefCell<Widget>>,
        mouseover: Rc<RefCell<AreaMouseover>>,
    ) {
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

    pub fn clear_mouse_state(&mut self) {
        self.hover_sprite = None;
        self.selection_box_start = None;
        self.path.clear();
        self.path_ap = None;
        Cursor::set_cursor_state(animation_state::Kind::Normal);
        self.clear_area_mouseover();
    }

    pub fn apply_theme(&mut self, theme: &Theme) {
        if let Some(ref image_id) = theme.custom.get("selection_box_image") {
            self.selection_box_image = ResourceSet::image(image_id);
        }

        if let Some(ref image_id) = theme.custom.get("path_point_image") {
            self.path_point_image = ResourceSet::image(image_id);
        }

        if let Some(ref image_id) = theme.custom.get("path_point_end_image") {
            self.path_point_end_image = ResourceSet::image(image_id);
        }
    }

    pub fn hover_sprite(&self) -> Option<&HoverSprite> {
        self.hover_sprite.as_ref()
    }

    pub fn get_path_draw_list(
        &self,
        x_offset: f32,
        y_offset: f32,
        millis: u32,
    ) -> Option<DrawList> {
        if !GameState::is_combat_active() {
            return None;
        }

        let image = match self.path_point_image {
            None => return None,
            Some(ref image) => image,
        };

        if self.path.is_empty() {
            return None;
        }

        let mut draw_list = DrawList::empty_sprite();

        for p in &self.path[0..self.path.len() - 1] {
            let x = p.0 - x_offset;
            let y = p.1 - y_offset;
            image.append_to_draw_list(
                &mut draw_list,
                &animation_state::NORMAL,
                x,
                y,
                1.0,
                1.0,
                millis,
            );
        }

        match self.path_point_end_image {
            None => (),
            Some(ref image) => {
                let last = self.path.last().unwrap();
                let x = last.0 - x_offset;
                let y = last.1 - y_offset;
                image.append_to_draw_list(
                    &mut draw_list,
                    &animation_state::NORMAL,
                    x,
                    y,
                    1.0,
                    1.0,
                    millis,
                );
            }
        }

        Some(draw_list)
    }

    pub fn draw_top(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        params: &Params,
        offset: (f32, f32),
        scale: (f32, f32),
        millis: u32,
    ) {
        if let Some(ref image) = self.selection_box_image {
            if let Some((x, y, x_end, y_end)) = self.get_selection_box_coords() {
                let w = x_end - x;
                let h = y_end - y;

                if w < 1.0 || h < 1.0 {
                    return;
                }

                let mut draw_list = DrawList::empty_sprite();
                image.append_to_draw_list(
                    &mut draw_list,
                    &animation_state::NORMAL,
                    x,
                    y,
                    w,
                    h,
                    millis,
                );
                renderer.draw(draw_list);
            }
        }

        if !GameState::is_combat_active() {
            return;
        }
        if let Some(ap) = self.path_ap {
            let font_rend = LineRenderer::new(&params.font);
            let text = format!("{} AP", Module::rules().format_ap(ap));
            let (x, y) = match &self.hover_sprite {
                None => (0.0, 0.0),
                Some(hover) => (
                    hover.x as f32 + offset.0,
                    hover.y as f32 + hover.h as f32 + offset.1,
                ),
            };

            let (mut draw_list, _) = font_rend.get_draw_list(&text, x, y, params.ap_scale);
            draw_list.set_color(params.ap_color);
            draw_list.set_scale(scale.0, scale.1);
            renderer.draw(draw_list);
        }
    }

    pub fn handle_left_drag(&mut self) {
        match self.selection_box_start {
            None => {
                self.selection_box_start = Some(Cursor::get_position_f32());
            }
            Some(_) => (),
        }
    }

    pub fn handle_left_click(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        scale: (f32, f32),
        scroll: (f32, f32),
    ) -> bool {
        let mut fire_action = false;
        if let Some((x, y, x_end, y_end)) = self.get_selection_box_coords() {
            let w = x_end - x;
            let h = y_end - y;
            if w < 1.0 && h < 1.0 {
                fire_action = true;
            } else {
                GameState::select_party_members(self.select_party_in_box(
                    &widget.borrow(),
                    scale,
                    scroll,
                ));
            }
            self.selection_box_start = None;
        } else {
            fire_action = true;
        }

        fire_action
    }

    pub fn on_mouse_exit(&mut self) {
        self.hover_sprite = None;
        self.selection_box_start = None;
        self.path.clear();
        self.path_ap = None;
    }
}
