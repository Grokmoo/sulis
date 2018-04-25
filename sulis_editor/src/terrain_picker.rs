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
use std::cell::RefCell;
use std::rc::Rc;

use rand::{self, Rng};

use sulis_core::config::CONFIG;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::{Module, area::tile::{Tile, TerrainRules, TerrainKind}};
use sulis_widgets::{Button, Label, Spinner};

use {AreaModel, EditorMode};

const NAME: &str = "terrain_picker";

pub struct TerrainPicker {
    cursor_sprite: Rc<Sprite>,

    cursor_pos: Option<Point>,
    brush_size: i32,
    grid_width: i32,
    grid_height: i32,

    terrain_rules: TerrainRules,
    terrain_kinds: Vec<TerrainKind>,

    cur_terrain: Option<TerrainKind>,
    cur_base_tile: Option<Rc<Tile>>,
}

impl TerrainPicker {
    pub fn new() -> Rc<RefCell<TerrainPicker>> {
        let cursor_sprite = match ResourceSet::get_sprite(&CONFIG.editor.cursor) {
            Err(_) => panic!("Unable to find cursor sprite '{}'", CONFIG.editor.cursor),
            Ok(sprite) => sprite,
        };

        let terrain_rules = Module::terrain_rules();
        let terrain_kinds = Module::terrain_kinds();

        Rc::new(RefCell::new(TerrainPicker {
            cursor_sprite,
            cursor_pos: None,
            brush_size: 4,
            grid_width: terrain_rules.grid_width as i32,
            grid_height: terrain_rules.grid_height as i32,
            terrain_rules,
            terrain_kinds,
            cur_terrain: None,
            cur_base_tile: None,
        }))
    }

    fn get_base_choices(&self, kind: &TerrainKind) -> Vec<Rc<Tile>> {
        let mut choices = Vec::new();
        let base_tile_id = format!("{}{}{}",self.terrain_rules.prefix, kind.id,
                                   self.terrain_rules.base_postfix);
        if let Some(tile) = Module::tile(&base_tile_id) {
            choices.push(tile);
        }
        for i in kind.variants.iter() {
            let tile_id = format!("{}{}{}{}", self.terrain_rules.prefix, kind.id,
                                  self.terrain_rules.variant_postfix, i.to_string());

            match Module::tile(&tile_id) {
                None => warn!("Variant tile '{}' not found", tile_id),
                Some(tile) => choices.push(tile),
            }
        }

        choices
    }

    fn gen_choice(&self, choices: &Vec<Rc<Tile>>) -> Rc<Tile> {
        if choices.len() == 1 { return Rc::clone(&choices[0]); }

        let base_weight = self.terrain_rules.base_weight;
        let total_weight = base_weight + choices.len() as u32;

        let roll = rand::thread_rng().gen_range(0, total_weight);

        if roll < base_weight {
            return Rc::clone(&choices[0]);
        }

        return Rc::clone(&choices[(roll - base_weight) as usize]);
    }
}

impl EditorMode for TerrainPicker {
    fn draw(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x_offset: f32, y_offset: f32,
            scale_x: f32, scale_y: f32, _millis: u32) {
        let gw = self.grid_width as f32;
        let gh = self.grid_height as f32;

        let mut draw_list = DrawList::empty_sprite();
        if let Some(pos) = self.cursor_pos {
            for y in 0..self.brush_size {
                for x in 0..self.brush_size {
                    let x = gw * x as f32 + pos.x as f32 + x_offset;
                    let y = gh * y as f32 + pos.y as f32 + y_offset;
                    draw_list.append(&mut DrawList::from_sprite_f32(&self.cursor_sprite, x, y, gw, gh));
                }
            }
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(Color::from_string("0F08"));
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (self.brush_size * self.grid_width, self.brush_size * self.grid_height)
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        let x = if x % 2 == 0 { x } else { x + 1 };
        let y = if y % 2 == 0 { y } else { y + 1 };
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let cur_terrain = match self.cur_terrain {
            None => return,
            Some(ref t) => t,
        };

        let choices = self.get_base_choices(cur_terrain);
        if choices.len() == 0 { return; }

        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_all_tiles(x_min, y_min, self.brush_size * self.grid_width,
                                self.brush_size * self.grid_height);

        for yi in 0..self.brush_size {
            for xi in 0..self.brush_size {
                let x = x_min + xi * self.grid_width;
                let y = y_min + yi * self.grid_height;

                model.add_tile(self.gen_choice(&choices), x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let x_min = if x % 2 == 0 { x } else { x + 1 };
        let y_min = if y % 2 == 0 { y } else { y + 1 };
        model.remove_all_tiles(x_min, y_min, self.brush_size * self.grid_width,
                                self.brush_size * self.grid_height);
    }
}

impl WidgetKind for TerrainPicker {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let brush_size = Widget::with_theme(Spinner::new(self.brush_size, 1, 20), "brush_size");
        brush_size.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.brush_size = spinner.value();
        })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let terrain_content = Widget::empty("terrain_content");
        for terrain_kind in self.terrain_kinds.iter() {
            let base_tile_id = format!("{}{}{}", self.terrain_rules.prefix, terrain_kind.id,
                                       self.terrain_rules.base_postfix);

            let tile = match Module::tile(&base_tile_id) {
                None => {
                    warn!("No base tile found for terrain kind '{}'", terrain_kind.id);
                    continue
                }, Some(tile) => tile,
            };

            let button = Widget::with_theme(Button::empty(), "terrain_button");
            button.borrow_mut().state.add_text_arg("icon", &tile.image_display.full_id());

            let terrain_kind = terrain_kind.clone();
            let cb: Callback = Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set terrain active: {}", terrain_kind.id);
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let parent = Widget::get_parent(&parent);
                    let terrain_picker = Widget::downcast_kind_mut::<TerrainPicker>(&parent);
                    terrain_picker.cur_base_tile = Some(Rc::clone(&tile));
                    terrain_picker.cur_terrain = Some(terrain_kind.clone());
                }
            }));
            button.borrow_mut().state.add_callback(cb);

            Widget::add_child_to(&terrain_content, button);
        }

        vec![brush_size, brush_size_label, terrain_content]
    }
}
