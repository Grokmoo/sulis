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

use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::{Color, Widget, WidgetKind};
use sulis_core::io::{DrawList, event, GraphicsRenderer};
use sulis_core::util::Point;

pub struct ColorButton {
    color: Color,
    icon: Option<Rc<Image>>,
}

impl ColorButton {
    pub fn new(color: Color) -> Rc<RefCell<ColorButton>> {
        Rc::new(RefCell::new(ColorButton {
            color,
            icon: None,
        }))
    }
}

impl WidgetKind for ColorButton {
    fn get_name(&self) -> &str { "color_button" }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if let Some(ref theme) = widget.theme {
            if let Some(ref icon_id) = theme.custom.get("icon") {
                self.icon = ResourceSet::get_image(icon_id);
            }
        }
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, millis: u32) {
        let icon = match self.icon {
            None => return,
            Some(ref icon) => icon,
        };

        let x = widget.state.inner_position.x as f32;
        let y = widget.state.inner_position.y as f32;
        let w = widget.state.inner_size.width as f32;
        let h = widget.state.inner_size.height as f32;
        let mut draw_list = DrawList::empty_sprite();
        icon.append_to_draw_list(&mut draw_list, &widget.state.animation_state,
                                 x, y, w, h, millis);
        draw_list.set_color(self.color);
        renderer.draw(draw_list);
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        Widget::fire_callback(widget, self);
        true
    }
}
