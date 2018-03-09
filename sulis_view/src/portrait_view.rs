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

use sulis_state::{ChangeListener, EntityState};
use sulis_core::ui::{Widget, WidgetKind};
use sulis_widgets::{Label};

pub const NAME: &str = "portrait_view";

pub struct PortraitView {
    entity: Rc<RefCell<EntityState>>,
}

impl PortraitView {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> Rc<RefCell<PortraitView>> {
        Rc::new(RefCell::new(PortraitView {
            entity,
        }))
    }
}

impl WidgetKind for PortraitView {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>>  {
        let mut entity = self.entity.borrow_mut();
        entity.actor.listeners.add(ChangeListener::invalidate(NAME, widget));

        let portrait = Widget::with_theme(Label::empty(), "portrait");
        if let Some(ref image) = entity.actor.actor.portrait {
            portrait.borrow_mut().state.add_text_arg("image", &image.id());
        }

        let ap_bar = Widget::with_theme(Label::empty(), "ap_bar");
        ap_bar.borrow_mut().state.add_text_arg("cur_ap", &entity.actor.ap().to_string());

        let hp_bar = Widget::with_theme(Label::empty(), "hp_bar");
        hp_bar.borrow_mut().state.add_text_arg("cur_hp", &entity.actor.hp().to_string());
        hp_bar.borrow_mut().state.add_text_arg("max_hp", &entity.actor.stats.max_hp.to_string());

        vec![portrait, ap_bar, hp_bar]
    }
}
