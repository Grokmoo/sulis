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

#[macro_use]
pub mod macros {
    #[macro_export]
    macro_rules! widget_kind {
        ($name:expr) => {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn get_name(&self) -> &str {
                $name
            }
        };
    }
}

pub mod animation_state;
pub use self::animation_state::AnimationState;

mod border;
pub use self::border::Border;

mod callback;
pub use self::callback::Callback;

pub mod color;
pub use self::color::Color;

mod cursor;
pub use self::cursor::Cursor;

mod font_renderer;
pub use self::font_renderer::FontRenderer;
pub use self::font_renderer::LineRenderer;

mod layout_kind;
pub use self::layout_kind::LayoutKind;

pub mod theme;
pub use self::theme::{Theme, ThemeSet};

pub mod theme_builder;
pub use self::theme_builder::{ThemeBuilder, ThemeBuilderSet};

pub mod widget;
pub use self::widget::Widget;

mod widget_kind;
pub(crate) use self::widget_kind::EmptyWidget;
pub use self::widget_kind::WidgetKind;

mod widget_state;
pub use self::widget_state::WidgetState;

use std::cell::RefCell;
use std::rc::Rc;

use crate::util::{Point, Size};

pub fn create_ui_tree(kind: Rc<RefCell<dyn WidgetKind>>) -> Rc<RefCell<Widget>> {
    debug!("Creating UI tree.");
    let root = Widget::with_defaults(kind);
    Widget::setup_root(&root);
    root
}

const SCALE_Y_BASE: f32 = 3200.0;
const SCALE_X_BASE: f32 = SCALE_Y_BASE * 16.0 / 9.0;

pub fn compute_area_scaling(pixel_size: Point) -> (f32, f32) {
    let scale_x = SCALE_X_BASE / (pixel_size.x as f32);
    let scale_y = SCALE_Y_BASE / (pixel_size.y as f32);

    (scale_x, scale_y)
}

#[derive(Copy, Clone)]
pub struct Scrollable {
    x: f32,
    y: f32,
    max_x: f32,
    max_y: f32,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
        }
    }
}

impl Scrollable {
    pub fn compute_max(
        &mut self,
        widget: &Widget,
        area_width: i32,
        area_height: i32,
        scale_x: f32,
        scale_y: f32,
    ) {
        self.max_x = (area_width as f32 - widget.state.inner_width() as f32 / scale_x).max(0.0);
        self.max_y = (area_height as f32 - widget.state.inner_height() as f32 / scale_y).max(0.0);
    }

    pub fn change(&mut self, delta_x: f32, delta_y: f32) {
        let x = self.x - delta_x;
        let y = self.y - delta_y;
        self.set(x, y);
    }

    pub fn set(&mut self, scroll_x: f32, scroll_y: f32) {
        let (x, y) = self.bound(scroll_x, scroll_y);

        self.x = x;
        self.y = y;
    }

    pub fn bound(&self, x: f32, y: f32) -> (f32, f32) {
        (x.clamp(0.0, self.max_x), y.clamp(0.0, self.max_y))
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }
}
