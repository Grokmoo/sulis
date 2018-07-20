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
use std::cell::RefCell;

use sulis_core::resource::ResourceSet;
use sulis_core::image::Image;
use sulis_core::ui::{LineRenderer, Widget, WidgetKind};
use sulis_core::io::{GraphicsRenderer};
use sulis_core::util::Point;

use Label;

const NAME: &str = "progress_bar";

pub struct ProgressBar {
    fraction: f32,
    bar: Option<Rc<Image>>,
    label: Rc<RefCell<Label>>,
}

impl ProgressBar {
    pub fn new(fraction: f32) -> Rc<RefCell<ProgressBar>> {
        Rc::new(RefCell::new(ProgressBar {
            bar: None,
            fraction: limit(fraction),
            label: Label::empty(),
        }))
    }

    pub fn set_fraction_filled(&mut self, frac: f32) {
        self.fraction = limit(frac);
    }
}

fn limit(fraction: f32) -> f32 {
    if fraction < 0.0 { return 0.0; }
    if fraction > 1.0 { return 1.0; }
    fraction
}

impl WidgetKind for ProgressBar {
    fn get_name(&self) -> &str { NAME }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }

        if let Some(ref theme) = widget.theme {
            if let Some(ref image_id) = theme.custom.get("bar_image") {
                self.bar = ResourceSet::get_image(image_id);
            }
        }
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
            widget: &Widget, millis: u32) {
        if self.fraction > 0.0 {
            if let Some(ref bar) = self.bar {
                let x = widget.state.inner_left() as f32;
                let y = widget.state.inner_top() as f32;
                let w = widget.state.inner_width() as f32 * self.fraction;
                let h = widget.state.inner_height() as f32;
                bar.draw(renderer, &widget.state.animation_state, x, y, w, h, millis);
            }
        }

        self.label.borrow_mut().draw(renderer, pixel_size, widget, millis);
    }
}
