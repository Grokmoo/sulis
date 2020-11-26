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
use std::time::Instant;

use sulis_core::config::Config;
use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{Font, ResourceSet};
use sulis_core::ui::{animation_state, Color, LineRenderer};
use sulis_core::util::{self, Offset, Point, Rect, Scale};
use sulis_module::{DamageKind, HitFlags, HitKind};

use crate::{AreaState, EntityState};

pub struct Params {
    pub font: Rc<Font>,
    pub scale: f32,
    pub ap_scale: f32,
    pub ap_color: Color,
    pub info_color: Color,
    pub miss_color: Color,
    pub hit_color: Color,
    pub heal_color: Color,
    pub damage_colors: [Color; 8],

    pub concealment_icon: Rc<dyn Image>,
    pub backstab_icon: Rc<dyn Image>,
    pub flanking_icon: Rc<dyn Image>,
    pub crit_icon: Rc<dyn Image>,
    pub hit_icon: Rc<dyn Image>,
    pub graze_icon: Rc<dyn Image>,
}

impl Default for Params {
    fn default() -> Params {
        use sulis_core::ui::color::*;
        Params {
            font: ResourceSet::default_font(),
            scale: 1.0,
            ap_scale: 1.0,
            ap_color: LIGHT_GRAY,
            info_color: LIGHT_GRAY,
            miss_color: LIGHT_GRAY,
            hit_color: RED,
            heal_color: BLUE,
            damage_colors: [
                LIGHT_GRAY, LIGHT_GRAY, LIGHT_GRAY, GREEN, CYAN, BLUE, YELLOW, PURPLE,
            ],
            concealment_icon: ResourceSet::empty_image(),
            backstab_icon: ResourceSet::empty_image(),
            flanking_icon: ResourceSet::empty_image(),
            crit_icon: ResourceSet::empty_image(),
            hit_icon: ResourceSet::empty_image(),
            graze_icon: ResourceSet::empty_image(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorKind {
    Info,
    Miss,
    Hit,
    Heal,
    Damage { kind: DamageKind },
}

struct Entry {
    text: String,
    icon: Option<IconKind>,
    color_kind: ColorKind,
}

#[derive(Copy, Clone)]
pub enum IconKind {
    Concealment,
    Backstab,
    Flanking,
    Crit,
    Hit,
    Graze,
}

pub struct AreaFeedbackText {
    area_pos: Point,
    pos_x: f32,
    pos_y: f32,
    start_time: Instant,
    duration: u32,
    move_rate: f32,

    hover_y: f32,
    alpha: f32,
    text_width: f32,

    total_text: String,
    entries: Vec<Entry>,
}

impl AreaFeedbackText {
    pub fn with_damage(
        target: &EntityState,
        area: &AreaState,
        hit_kind: HitKind,
        hit_flags: HitFlags,
        damage: &[(DamageKind, u32)],
    ) -> AreaFeedbackText {
        let mut text = AreaFeedbackText::with_target(target, area);

        if hit_flags.sneak_attack {
            text.add_icon_entry(IconKind::Backstab, ColorKind::Info);
        } else if hit_flags.flanking {
            text.add_icon_entry(IconKind::Flanking, ColorKind::Info);
        }

        if hit_flags.concealment {
            text.add_icon_entry(IconKind::Concealment, ColorKind::Info);
        }

        let mut first = true;
        for (kind, amount) in damage {
            if !first {
                text.add_entry(" + ".to_string(), ColorKind::Info);
            }

            let color = ColorKind::Damage { kind: *kind };
            text.add_entry(format!("{}", amount), color);

            first = false;
        }

        match hit_kind {
            HitKind::Graze => text.add_icon_entry(IconKind::Graze, ColorKind::Info),
            HitKind::Hit => text.add_icon_entry(IconKind::Hit, ColorKind::Info),
            HitKind::Crit => text.add_icon_entry(IconKind::Crit, ColorKind::Info),
            HitKind::Miss => text.add_entry("Miss".to_string(), ColorKind::Miss),
            HitKind::Auto => (),
        }

        text
    }

    pub fn with_target(target: &EntityState, area: &AreaState) -> AreaFeedbackText {
        let move_rate = 3.0;
        let mut area_pos = target.location.to_point();
        loop {
            let mut valid = true;

            let pos_y = area_pos.y as f32;
            for text in area.feedback_text_iter() {
                let text_pos_y = text.area_pos().y as f32 - text.cur_hover_y();
                if (pos_y - text_pos_y).abs() < 0.7 {
                    area_pos.y -= 1;
                    valid = false;
                    break;
                }
            }

            if valid || area_pos.y == 0 {
                break;
            }
        }

        let width = target.size.width as f32;
        let pos_x = area_pos.x as f32 + width / 2.0;
        let pos_y = area_pos.y as f32 - 1.5;

        AreaFeedbackText::new(area_pos, pos_x, pos_y, move_rate)
    }

    pub fn new(area_pos: Point, pos_x: f32, pos_y: f32, move_rate: f32) -> AreaFeedbackText {
        AreaFeedbackText {
            area_pos,
            total_text: String::new(),
            text_width: 0.0,
            pos_x,
            pos_y,
            move_rate,
            start_time: Instant::now(),
            duration: Config::animation_base_time_millis() * 50,
            hover_y: 0.0,
            alpha: 1.0,
            entries: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add_icon_entry(&mut self, icon: IconKind, color_kind: ColorKind) {
        self.total_text.push('w');
        self.entries.push(Entry {
            text: String::new(),
            icon: Some(icon),
            color_kind,
        });
    }

    pub fn add_entry(&mut self, text: String, color_kind: ColorKind) {
        self.duration += text.len() as u32 / 2;
        self.total_text.push_str(&text);
        self.entries.push(Entry {
            text,
            color_kind,
            icon: None,
        });
    }

    pub fn area_pos(&self) -> Point {
        self.area_pos
    }

    pub fn cur_hover_y(&self) -> f32 {
        self.hover_y
    }

    pub fn update(&mut self) {
        let frac =
            util::get_elapsed_millis(self.start_time.elapsed()) as f32 / self.duration as f32;

        self.hover_y = frac * self.move_rate;

        if frac < 0.5 {
            self.alpha = 1.0;
        } else {
            self.alpha = (1.0 - frac) * 2.0;
        }
    }

    pub fn retain(&self) -> bool {
        self.alpha > 0.0
    }

    // it is assumed that the params being passed in here do not change
    pub fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        params: &Params,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        // creating the line renderer here is not ideal but is a low cost operation
        let font_renderer = LineRenderer::new(&params.font);
        if self.text_width == 0.0 {
            self.text_width =
                params.font.get_width(&self.total_text) as f32 / params.font.line_height as f32;
        }

        let mut pos_x = offset.x + self.pos_x - params.scale * self.text_width / 2.0;
        if pos_x < 0.0 {
            pos_x = 0.0;
        }
        let pos_y = offset.y + self.pos_y - self.hover_y;

        for entry in &self.entries {
            let mut color = match entry.color_kind {
                ColorKind::Info => params.info_color,
                ColorKind::Miss => params.miss_color,
                ColorKind::Hit => params.hit_color,
                ColorKind::Heal => params.heal_color,
                ColorKind::Damage { kind } => {
                    let index = kind.index();
                    params.damage_colors[index]
                }
            };
            color.a *= self.alpha;

            if let Some(icon) = entry.icon {
                let w = params.scale / 1.5;
                let h = params.scale / 1.5;

                let state = &animation_state::NORMAL;
                let image = match icon {
                    IconKind::Concealment => &params.concealment_icon,
                    IconKind::Backstab => &params.backstab_icon,
                    IconKind::Flanking => &params.flanking_icon,
                    IconKind::Crit => &params.crit_icon,
                    IconKind::Hit => &params.hit_icon,
                    IconKind::Graze => &params.graze_icon,
                };

                let rect = Rect {
                    x: pos_x,
                    y: pos_y + params.scale * 0.15,
                    w,
                    h,
                };
                let mut draw_list = DrawList::empty_sprite();
                image.append_to_draw_list(&mut draw_list, state, rect, millis);
                draw_list.set_scale(scale);
                draw_list.set_color(color);
                renderer.draw(draw_list);

                pos_x += 1.5 * params.scale / params.font.line_height as f32
                    * params.font.get_char_width('w') as f32;
            } else {
                let offset = Offset { x: pos_x, y: pos_y };
                let (mut draw_list, next_x) =
                    font_renderer.get_draw_list(&entry.text, offset, params.scale);
                draw_list.set_scale(scale);
                draw_list.set_color(color);
                renderer.draw(draw_list);
                pos_x = next_x;
            }
        }
    }
}
