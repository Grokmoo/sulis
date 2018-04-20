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

use sulis_module::{Cutscene};
use sulis_core::io::event;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, Label, TextArea};

pub const NAME: &str = "cutscene_window";

pub struct CutsceneWindow {
    cutscene: Rc<Cutscene>,
    frame_index: usize,
}

impl CutsceneWindow {
    pub fn new(cutscene: Rc<Cutscene>) -> Rc<RefCell<CutsceneWindow>> {
        Rc::new(RefCell::new(CutsceneWindow {
            cutscene,
            frame_index: 0
        }))
    }
}

impl WidgetKind for CutsceneWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let frame = self.cutscene.frames.get(self.frame_index);
        let frame = match frame {
            None => {
                widget.borrow_mut().mark_for_removal();
                return Vec::new();
            }, Some(ref frame) => frame,
        };

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let click_label = Widget::with_theme(Label::empty(), "click_label");

        let text_area = Widget::with_defaults(TextArea::empty());

        text_area.borrow_mut().state.add_text_arg("0", &frame.text);

        vec![close, click_label, text_area]
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        widget.borrow_mut().invalidate_children();
        self.frame_index += 1;
        true
    }
}
