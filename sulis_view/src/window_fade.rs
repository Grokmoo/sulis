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

use sulis_core::image::Image;
use sulis_core::io::{event::ClickKind, DrawList, GraphicsRenderer};
use sulis_core::resource::ResourceSet;
use sulis_core::ui::{animation_state, Widget, WidgetKind};
use sulis_core::util::Point;

pub enum Mode {
    In,
    OutIn,
}

pub struct WindowFade {
    elapsed_millis: u32,
    fade_millis: u32,
    pause_millis: u32,
    fill: Option<Rc<Image>>,
    frac: f32,

    mode: Mode,
}

impl WindowFade {
    pub fn new(mode: Mode) -> Rc<RefCell<WindowFade>> {
        let frac = match mode {
            Mode::In => 1.0,
            Mode::OutIn => 0.0,
        };

        Rc::new(RefCell::new(WindowFade {
            elapsed_millis: 0,
            fade_millis: 1000,
            pause_millis: 1000,
            fill: None,
            frac,
            mode,
        }))
    }
}

impl WidgetKind for WindowFade {
    widget_kind!["window_fade"];

    fn update(&mut self, widget: &Rc<RefCell<Widget>>, millis: u32) {
        if self.elapsed_millis == 0 {
            // don't count the first frame in case we are loading
            self.elapsed_millis = 1;
            return;
        }

        self.elapsed_millis += millis;

        let total = match self.mode {
            Mode::In => self.fade_millis,
            Mode::OutIn => 2 * self.fade_millis + self.pause_millis,
        };

        let elapsed = self.elapsed_millis;
        if elapsed > total {
            self.frac = 0.0;
            widget.borrow_mut().mark_for_removal();
        } else {
            self.frac = match self.mode {
                Mode::In => 1.0 - elapsed as f32 / total as f32,
                Mode::OutIn => {
                    if elapsed > self.fade_millis + self.pause_millis {
                        let rel = elapsed - (self.fade_millis + self.pause_millis);
                        1.0 - rel as f32 / self.fade_millis as f32
                    } else if elapsed > self.fade_millis {
                        1.0
                    } else {
                        elapsed as f32 / self.fade_millis as f32
                    }
                }
            };
        }
    }

    fn layout(&mut self, widget: &mut Widget) {
        let theme = &widget.theme;
        self.fade_millis = theme.get_custom_or_default("fade_millis", 1000);
        self.pause_millis = theme.get_custom_or_default("pause_millis", 1000);

        if let Some(ref image_id) = theme.custom.get("fill_image") {
            self.fill = ResourceSet::image(image_id);
        }
        widget.do_base_layout();
    }

    fn draw(
        &mut self,
        renderer: &mut GraphicsRenderer,
        _pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
        if let Some(ref fill) = self.fill {
            let x = widget.state.inner_left() as f32;
            let y = widget.state.inner_top() as f32;
            let w = widget.state.inner_width() as f32;
            let h = widget.state.inner_height() as f32;
            let mut draw_list = DrawList::empty_sprite();
            fill.append_to_draw_list(&mut draw_list, &animation_state::NORMAL, x, y, w, h, millis);

            draw_list.set_alpha(self.frac);
            renderer.draw(draw_list);
        }
    }

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
}
