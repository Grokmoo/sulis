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
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::util::{Offset, Point, Scale};
use sulis_state::{ChangeListener, EntityState, GameState};

pub const NAME: &str = "initiative_ticker";

pub struct InitiativeTicker {}

impl InitiativeTicker {
    pub fn new() -> Rc<RefCell<InitiativeTicker>> {
        Rc::new(RefCell::new(InitiativeTicker {}))
    }
}

impl WidgetKind for InitiativeTicker {
    widget_kind!(NAME);

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_enter(widget);
        false
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

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_enabled(false);

        let mgr = GameState::turn_manager();
        mgr.borrow_mut()
            .listeners
            .add(ChangeListener::invalidate(NAME, widget));

        let pane = Widget::empty("pane");
        let mut first = true;
        for entity in mgr.borrow().active_iter() {
            let theme = if first { "current_entry" } else { "entry" };
            let widget = Widget::with_theme(TickerLabel::new(&entity), theme);
            Widget::add_child_to(&pane, widget);
            first = false;
        }

        vec![pane]
    }
}

struct TickerLabel {
    entity: Rc<RefCell<EntityState>>,
}

impl TickerLabel {
    fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<TickerLabel>> {
        Rc::new(RefCell::new(TickerLabel {
            entity: Rc::clone(entity),
        }))
    }
}

impl WidgetKind for TickerLabel {
    widget_kind!(NAME);

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        _pixel_size: Point,
        widget: &Widget,
        _millis: u32,
    ) {
        let entity = self.entity.borrow();

        let x = widget.state.inner_left() as f32;
        let y = widget.state.inner_top() as f32;

        let inner_width = widget.state.inner_width() as f32;
        let inner_height = widget.state.inner_height() as f32;

        let w = inner_width / (entity.size.width as f32 + 2.0);
        let h = inner_height / (entity.size.height as f32 + 2.0);
        let size = if w > h { h } else { w };

        let cx = x + (inner_width - size) / 2.0 - 2.0 + entity.actor.actor.race.ticker_offset.0;
        let cy = y + (inner_height - size) / 2.0 - 2.0 + entity.actor.actor.race.ticker_offset.1;

        let offset = Offset {
            x: cx / size,
            y: cy / size,
        };
        let scale = Scale { x: size, y: size };

        self.entity
            .borrow()
            .draw_no_pos(renderer, offset, scale, 1.0);
    }
}
