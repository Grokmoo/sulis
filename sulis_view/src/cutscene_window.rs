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
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, TextArea};
use sulis_state::GameState;

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
                if let Some(cb) = &self.cutscene.on_end {
                    let pc = GameState::player();
                    GameState::add_ui_callback(cb.clone(), &pc, &pc);
                }
                return Vec::new();
            }, Some(ref frame) => frame,
        };

        let close = Widget::with_theme(Button::empty(), "close");

        let cutscene = Rc::clone(&self.cutscene);
        close.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
            let parent = Widget::get_parent(widget);
            parent.borrow_mut().mark_for_removal();

            if let Some(cb) = &cutscene.on_end {
                let pc = GameState::player();
                GameState::add_ui_callback(cb.clone(), &pc, &pc);
            }
        })));

        let next_button = Widget::with_theme(Button::empty(), "next_button");
        next_button.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, _| {
            let parent = Widget::get_parent(&widget);
            let window = Widget::downcast_kind_mut::<CutsceneWindow>(&parent);
            window.frame_index += 1;
            parent.borrow_mut().invalidate_children();
        })));

        let text_area = Widget::with_defaults(TextArea::empty());

        text_area.borrow_mut().state.add_text_arg("0", &frame.text);

        vec![close, text_area, next_button]
    }
}
