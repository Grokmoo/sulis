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

use sulis_core::io::InputActionKind;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{InputField, Label};
use sulis_state::{GameState, ScriptState};

pub const NAME: &str = "console_window";

pub struct ConsoleWindow {
    input: Rc<RefCell<InputField>>,
    input_widget: Rc<RefCell<Widget>>,
    output: Rc<RefCell<Widget>>,
    history: Vec<String>,
    history_index: usize,
    script_state: ScriptState,
}

impl ConsoleWindow {
    pub fn new() -> Rc<RefCell<ConsoleWindow>> {
        let input = InputField::new("");
        Rc::new(RefCell::new(ConsoleWindow {
            input: Rc::clone(&input),
            input_widget: Widget::with_theme(input, "input"),
            output: Widget::with_theme(Label::empty(), "output"),
            history: Vec::new(),
            history_index: 0,
            script_state: ScriptState::default(),
        }))
    }

    pub fn execute_script(&mut self, script: String) {
        if script.trim().is_empty() {
            return;
        }

        self.history.push(script[0..script.len() - 1].to_string());
        self.history_index = self.history.len();

        let party = GameState::party();

        let result = match self.script_state.console(script, &party) {
            Ok(result) => result,
            Err(rlua::Error::FromLuaConversionError { .. }) => "Success".to_string(),
            Err(e) => format!("{}", e),
        };

        info!("Console result: {}", result);
        self.output.borrow_mut().state.text = result;
    }

    pub fn current_history_text(&self) -> String {
        if self.history_index >= self.history.len() {
            return "".to_string();
        }

        self.history[self.history_index].clone()
    }

    pub fn grab_keyboard(&self) {
        Widget::grab_keyboard_focus(&self.input_widget);
        self.input.borrow_mut().set_ignore_next();
    }
}

impl WidgetKind for ConsoleWindow {
    widget_kind!(NAME);

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let prompt = Widget::with_theme(Label::empty(), "prompt");

        self.input
            .borrow_mut()
            .set_key_press_callback(Rc::new(|widget, field, key| {
                let (parent, console) = Widget::parent_mut::<ConsoleWindow>(widget);
                match key {
                    InputActionKind::ToggleConsole => {
                        Widget::clear_keyboard_focus(widget);
                        parent.borrow_mut().state.set_visible(false);
                    }
                    InputActionKind::ConsoleHistoryPrevious => {
                        if console.history_index > 0 {
                            console.history_index -= 1;
                            field.set_text(&console.current_history_text(), widget);
                        }
                    }
                    InputActionKind::ConsoleHistoryNext => {
                        if console.history_index < console.history.len() {
                            console.history_index += 1;
                            field.set_text(&console.current_history_text(), widget);
                        }
                    }
                    _ => (),
                }
            }));

        self.input
            .borrow_mut()
            .set_enter_callback(Callback::new(Rc::new(|widget, kind| {
                let field = match kind.as_any_mut().downcast_mut::<InputField>() {
                    None => panic!(),
                    Some(widget) => widget,
                };

                let text = field.text();
                field.clear(widget);

                let (_, console) = Widget::parent_mut::<ConsoleWindow>(widget);
                console.execute_script(text);
            })));

        vec![prompt, self.input_widget.clone(), self.output.clone()]
    }
}
