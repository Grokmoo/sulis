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

use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::io::{GraphicsRenderer};
use sulis_core::util::Point;
use sulis_widgets::{MarkupRenderer, TextArea};
use sulis_state::{ChangeListener, EntityState};

const NAME: &'static str = "entity_mouseover";

pub struct EntityMouseover {
    entity: Rc<RefCell<EntityState>>,
    text_area: Rc<RefCell<TextArea>>,
}

impl EntityMouseover {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<RefCell<EntityMouseover>> {
        Rc::new(RefCell::new(EntityMouseover {
            entity: Rc::clone(entity),
            text_area: TextArea::empty(),
        }))
    }
}

impl WidgetKind for EntityMouseover {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_remove(&mut self) {
        //self.entity.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate_layout(NAME, widget));

        Vec::new()
    }

    fn layout(&self, widget: &mut Widget) {
        info!("Layout entity mouseover");
        widget.state.add_text_arg("name", &self.entity.borrow().actor.actor.name);
        widget.state.add_text_arg("cur_hp", &self.entity.borrow().actor.hp().to_string());
        widget.state.add_text_arg("max_hp", &self.entity.borrow().actor.stats.max_hp.to_string());

        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(
                    MarkupRenderer::new(font, widget.state.inner_size.width)));
        }
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.text_area.borrow_mut().draw_graphics_mode(renderer, pixel_size, widget, millis);
    }
}
