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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

pub mod animation_state;
pub use self::animation_state::AnimationState;

mod border;
pub use self::border::Border;

mod button;
pub use self::button::Button;

mod callback;
pub use self::callback::Callback;

pub mod color;
pub use self::color::Color;

mod confirmation_window;
pub use self::confirmation_window::ConfirmationWindow;

mod cursor;
pub use self::cursor::Cursor;

mod font_renderer;
pub use self::font_renderer::FontRenderer;
pub use self::font_renderer::LineRenderer;

mod label;
pub use self::label::Label;

mod layout_kind;
pub use self::layout_kind::LayoutKind;

pub mod list_box;
pub use self::list_box::ListBox;

pub mod markup_renderer;
pub use self::markup_renderer::MarkupRenderer;

pub mod text_area;
pub use self::text_area::TextArea;

pub mod theme;
pub use self::theme::Theme;

pub mod widget;
pub use self::widget::Widget;

mod widget_kind;
pub use self::widget_kind::WidgetKind;
pub use self::widget_kind::EmptyWidget;

mod widget_state;
pub use self::widget_state::WidgetState;

use std::rc::Rc;
use std::cell::RefCell;

use config::CONFIG;
use resource::ResourceSet;
use util::Size;

pub fn create_ui_tree(kind: Rc<WidgetKind>) -> Rc<RefCell<Widget>> {

    debug!("Creating UI tree.");
    let root = Widget::with_defaults(kind);
    root.borrow_mut().state.set_size(Size::new(CONFIG.display.width,
                                               CONFIG.display.height));
    root.borrow_mut().theme = Some(ResourceSet::get_theme());

    root
}
