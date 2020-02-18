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

use crate::image::Image;
use crate::io::{event::ClickKind, DrawList, GraphicsRenderer};
use crate::resource::ResourceSet;
use crate::ui::{theme, LineRenderer, Widget, WidgetKind};
use crate::util::{Point, Rect};
use crate::widget_kind;
use crate::widgets::{Label, TextArea};

const NAME: &str = "progress_bar";

pub struct ProgressBar {
    fraction: f32,
    bar: Option<Rc<dyn Image>>,
    label: Rc<RefCell<Label>>,
    tooltip: String,
}

impl ProgressBar {
    pub fn new(fraction: f32) -> Rc<RefCell<ProgressBar>> {
        Rc::new(RefCell::new(ProgressBar {
            bar: None,
            fraction: limit(fraction),
            label: Label::empty(),
            tooltip: String::new(),
        }))
    }

    pub fn set_fraction_filled(&mut self, frac: f32) {
        self.fraction = limit(frac);
    }
}

fn limit(fraction: f32) -> f32 {
    if fraction < 0.0 {
        return 0.0;
    }
    if fraction > 1.0 {
        return 1.0;
    }
    fraction
}

impl WidgetKind for ProgressBar {
    widget_kind![NAME];

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(
        &mut self,
        _widget: &Rc<RefCell<Widget>>,
        _kind: ClickKind,
        _delta_x: f32,
        _delta_y: f32,
    ) -> bool {
        false
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    // TODO refactor tooltip code into a common case somewhere
    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        if self.tooltip.is_empty() {
            return false;
        }

        let tooltip = Widget::with_theme(TextArea::empty(), "tooltip");
        tooltip.borrow_mut().state.add_text_arg("0", &self.tooltip);

        let (x, y) = {
            let state = &widget.borrow().state;
            (state.position().x, state.position().y)
        };

        Widget::set_mouse_over_widget(widget, tooltip, x, y);

        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }

        if let Some(ref image_id) = widget.theme.custom.get("bar_image") {
            self.bar = ResourceSet::image(image_id);
        }

        if let Some(tooltip) = widget.theme.custom.get("tooltip") {
            self.tooltip = theme::expand_text_args(tooltip, &widget.state);
        }
    }

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
        if self.fraction > 0.0 {
            if let Some(ref bar) = self.bar {
                let rect = Rect {
                    x: widget.state.inner_left() as f32,
                    y: widget.state.inner_top() as f32,
                    w: widget.state.inner_width() as f32 * self.fraction,
                    h: widget.state.inner_height() as f32,
                };

                let mut draw_list = DrawList::empty_sprite();
                bar.append_to_draw_list(
                    &mut draw_list,
                    &widget.state.animation_state,
                    rect,
                    millis,
                );

                // draw only the fractional part for images consisting of only
                // a single quad
                if draw_list.quads.len() == 6 {
                    // info!("Drawlist for prog bar initial: {:#?}", draw_list);
                    let tcx_min = draw_list.quads[1].tex_coords[0];
                    let tcx_max = draw_list.quads[2].tex_coords[0];

                    let new_x_max = tcx_min + (tcx_max - tcx_min) * self.fraction;

                    draw_list.quads[2].tex_coords[0] = new_x_max;
                    draw_list.quads[3].tex_coords[0] = new_x_max;
                    draw_list.quads[5].tex_coords[0] = new_x_max;

                    // info!("Drawlist for prog bar mod: {:#?}", draw_list);
                }

                renderer.draw(draw_list);
            }
        }

        self.label
            .borrow_mut()
            .draw(renderer, pixel_size, widget, millis);
    }
}
