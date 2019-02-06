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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, TextArea};
use sulis_module::Cutscene;
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
            frame_index: 0,
        }))
    }
}

pub fn add_on_end_cbs(cutscene: &Rc<Cutscene>) {
    if cutscene.on_end.len() > 0 {
        let pc = GameState::player();
        GameState::add_ui_callback(cutscene.on_end.clone(), &pc, &pc);
    }
}

impl WidgetKind for CutsceneWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let frame = self.cutscene.frames.get(self.frame_index);
        let frame = match frame {
            None => {
                widget.borrow_mut().mark_for_removal();
                add_on_end_cbs(&self.cutscene);
                return Vec::new();
            }
            Some(ref frame) => frame,
        };

        let close = Widget::with_theme(Button::empty(), "close");

        let cutscene = Rc::clone(&self.cutscene);
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let (parent, _) = Widget::parent_mut::<CutsceneWindow>(widget);
                parent.borrow_mut().mark_for_removal();

                add_on_end_cbs(&cutscene);
            })));

        let next_button = Widget::with_theme(Button::empty(), "next_button");
        next_button
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, window) = Widget::parent_mut::<CutsceneWindow>(widget);
                window.frame_index += 1;
                parent.borrow_mut().invalidate_children();
            })));

        let text_area = Widget::with_defaults(TextArea::empty());

        text_area.borrow_mut().state.add_text_arg("0", &frame.text);

        vec![close, text_area, next_button]
    }
}
