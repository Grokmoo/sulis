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

use sulis_core::io::event::ClickKind;
use sulis_core::io::{GraphicsRenderer, InputAction};
use sulis_core::ui::{compute_area_scaling, Cursor, Scrollable, Widget, WidgetKind};
use sulis_core::util::{Scale, Offset, Point};
use sulis_module::area::MAX_AREA_SIZE;

use crate::{AreaModel, EditorMode};

const NAME: &str = "area_editor";

type EditorModeRef = Rc<RefCell<dyn EditorMode>>;

pub struct AreaEditor {
    cur_editor: Option<EditorModeRef>,
    pub(crate) model: AreaModel,

    scroll: Scrollable,
    scale: (f32, f32),

    last_click_position: Option<Point>,
}

impl AreaEditor {
    pub fn new() -> Rc<RefCell<AreaEditor>> {
        Rc::new(RefCell::new(AreaEditor {
            model: AreaModel::default(),
            cur_editor: None,
            scroll: Scrollable::default(),
            scale: (1.0, 1.0),
            last_click_position: None,
        }))
    }

    pub fn clear_area(&mut self) {
        self.model = AreaModel::default();
        self.scroll = Scrollable::default();
        self.cur_editor = None;
    }

    pub fn set_editor(&mut self, editor: EditorModeRef) {
        self.cur_editor = Some(editor);
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>, width: i32, height: i32) -> (i32, i32) {
        let mut x = Cursor::get_x_f32() - widget.borrow().state.inner_left() as f32;
        let mut y = Cursor::get_y_f32() - widget.borrow().state.inner_top() as f32;

        x /= self.scale.0;
        y /= self.scale.1;
        x -= width as f32 / 2.0;
        y -= height as f32 / 2.0;

        (
            (x + self.scroll.x()).round() as i32,
            (y + self.scroll.y()).round() as i32,
        )
    }

    fn get_event_data(
        &self,
        widget: &Rc<RefCell<Widget>>,
    ) -> Option<(EditorModeRef, i32, i32)> {
        let editor = match self.cur_editor {
            None => return None,
            Some(ref editor) => editor,
        };

        let (width, height) = editor.borrow().cursor_size();
        let (x, y) = self.get_cursor_pos(widget, width, height);
        Some((Rc::clone(editor), x, y))
    }
}

impl WidgetKind for AreaEditor {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
        self.scale = compute_area_scaling(pixel_size);
        let scale = Scale { x: self.scale.0, y: self.scale.1 };

        let p = widget.state.position();
        // TODO fix this hack
        let p = Point::new(p.x / 4, p.y / 4);

        self.model.draw(
            renderer,
            Offset {
                x: p.x as f32 - self.scroll.x(),
                y: p.y as f32 - self.scroll.y(),
            },
            scale,
            millis,
        );

        if let Some(ref editor) = self.cur_editor {
            let mut editor = editor.borrow_mut();
            editor.draw_mode(
                renderer,
                &self.model,
                Offset {
                    x: p.x as f32 - self.scroll.x(),
                    y: p.y as f32 - self.scroll.y(),
                },
                scale,
                millis,
            );
        }
    }

    fn on_key_press(&mut self, _widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        let delta = match key {
            InputAction::ZoomIn => 1,
            InputAction::ZoomOut => -1,
            _ => return false,
        };

        let editor = match self.cur_editor {
            None => return true,
            Some(ref editor) => editor,
        };

        editor.borrow_mut().mouse_scroll(&mut self.model, delta);

        true
    }

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        let (editor, x, y) = match self.get_event_data(widget) {
            None => return true,
            Some(value) => value,
        };

        self.last_click_position = Some(Point::new(x, y));
        match kind {
            ClickKind::Primary => editor.borrow_mut().left_click(&mut self.model, x, y),
            ClickKind::Secondary => editor.borrow_mut().right_click(&mut self.model, x, y),
            _ => (),
        }

        true
    }

    fn on_mouse_release(&mut self, _: &Rc<RefCell<Widget>>, _: ClickKind) -> bool {
        self.last_click_position = None;
        true
    }

    fn on_mouse_drag(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        kind: ClickKind,
        delta_x: f32,
        delta_y: f32,
    ) -> bool {
        if let ClickKind::Tertiary = kind {
            self.scroll.compute_max(
                &*widget.borrow(),
                MAX_AREA_SIZE,
                MAX_AREA_SIZE,
                self.scale.0,
                self.scale.1,
            );
            self.scroll.change(delta_x, delta_y);
            return true;
        }

        let (editor, x, y) = match self.get_event_data(widget) {
            None => return true,
            Some(value) => value,
        };

        // only fire left / right click event if the position has changed
        match self.last_click_position {
            None => (),
            Some(p) => {
                if p.x == x && p.y == y {
                    return true;
                }
            }
        }

        self.last_click_position = Some(Point::new(x, y));
        match kind {
            ClickKind::Primary => editor.borrow_mut().left_click(&mut self.model, x, y),
            ClickKind::Secondary => editor.borrow_mut().right_click(&mut self.model, x, y),
            _ => (),
        }

        true
    }

    fn on_mouse_move(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        _delta_x: f32,
        _delta_y: f32,
    ) -> bool {
        let (editor, x, y) = match self.get_event_data(widget) {
            None => return true,
            Some(value) => value,
        };
        editor.borrow_mut().mouse_move(&mut self.model, x, y);

        true
    }
}
