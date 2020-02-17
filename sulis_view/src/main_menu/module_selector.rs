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
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::ui::*;
use sulis_core::util::ActiveResources;
use sulis_core::widgets::{Button, Label, ScrollDirection, ScrollPane, TextArea};
use sulis_module::ModuleInfo;
use sulis_state::NextGameStep;

use crate::main_menu::MainMenu;
use crate::LoadingScreen;

pub struct ModuleSelector {
    modules: Vec<ModuleInfo>,
    selected_module: Option<usize>,
}

impl ModuleSelector {
    pub fn new(modules: Vec<ModuleInfo>) -> Rc<RefCell<ModuleSelector>> {
        Rc::new(RefCell::new(ModuleSelector {
            modules,
            selected_module: None,
        }))
    }
}

impl WidgetKind for ModuleSelector {
    widget_kind!("module_selector");

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");
        let modules_title = Widget::with_theme(Label::empty(), "modules_title");
        let play = Widget::with_theme(Button::empty(), "play_button");

        let details = Widget::with_theme(TextArea::empty(), "details");

        if let Some(index) = self.selected_module {
            details
                .borrow_mut()
                .state
                .add_text_arg("description", &self.modules[index].description);
        }

        let mut groups = HashMap::new();
        for (index, module) in self.modules.iter().enumerate() {
            let vec = groups.entry(module.group.clone()).or_insert_with(Vec::new);
            vec.push(index);
        }

        for (_, modules) in groups.iter_mut() {
            modules.sort_by_key(|m| self.modules[*m].group.position);
        }

        let mut campaign_groups: Vec<_> = groups.keys().collect();
        campaign_groups.sort();
        campaign_groups.dedup();

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        let scroll_widget = Widget::with_theme(scrollpane.clone(), "campaign_groups");
        for campaign_group in campaign_groups {
            let group_widget = Widget::empty("campaign_group");

            let mut modules_in_group = 0;
            for index in groups.get(campaign_group).iter().flat_map(|v| v.iter()) {
                let index = *index;
                let module = &self.modules[index];

                let button = Widget::with_theme(Button::empty(), "module_button");
                button
                    .borrow_mut()
                    .state
                    .add_text_arg("module", &module.name);
                if let Some(selected_index) = self.selected_module {
                    button
                        .borrow_mut()
                        .state
                        .set_active(index == selected_index);
                }

                button
                    .borrow_mut()
                    .state
                    .add_callback(Callback::new(Rc::new(move |widget, _| {
                        let (parent, module_selector) =
                            Widget::parent_mut::<ModuleSelector>(widget);
                        module_selector.selected_module = Some(index);
                        parent.borrow_mut().invalidate_children();
                    })));
                Widget::add_child_to(&group_widget, button);
                modules_in_group += 1;
            }

            if modules_in_group > 1 {
                // only show group label if there is more than one campaign in the group
                let name = Widget::with_theme(Label::empty(), "name_label");
                name.borrow_mut()
                    .state
                    .add_text_arg("name", &campaign_group.name);
                Widget::add_child_to_front(&group_widget, name);
            }
            scrollpane.borrow().add_to_content(group_widget);
        }

        play.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let module = {
                    let (_, module_selector) = Widget::parent_mut::<ModuleSelector>(widget);

                    let index = match module_selector.selected_module {
                        None => return,
                        Some(index) => index,
                    };
                    module_selector.modules[index].clone()
                };

                let (root, menu) = Widget::parent_mut::<MainMenu>(widget);
                let mut active = ActiveResources::read();
                active.campaign = Some(module.dir);
                active.write();
                menu.next_step = Some(NextGameStep::MainMenuReloadResources);

                let loading_screen = Widget::with_defaults(LoadingScreen::new());
                loading_screen.borrow_mut().state.set_modal(true);
                Widget::add_child_to(&root, loading_screen);
            })));
        play.borrow_mut()
            .state
            .set_enabled(self.selected_module.is_some());

        vec![title, modules_title, play, scroll_widget, details]
    }
}
