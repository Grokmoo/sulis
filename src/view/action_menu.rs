use std::rc::Rc;
use std::cell::RefCell;

use grt::resource::area::Transition;
use grt::ui::{Button, Callback, Cursor, Label, list_box, ListBox, Widget, WidgetKind};
use grt::io::event::ClickKind;
use grt::util::Point;

use state::{AreaState, GameState, EntityState};

pub struct ActionMenu {
    area_state: Rc<RefCell<AreaState>>,
    hovered_entity: Option<Rc<RefCell<EntityState>>>,
    is_hover_pc: bool,
    area_pos: Point,
}

impl ActionMenu {
    pub fn new(area_state: Rc<RefCell<AreaState>>, x: i32, y: i32) -> Rc<ActionMenu> {
        let (hovered_entity, is_hover_pc) =
            if let Some(ref entity) = area_state.borrow().get_entity_at(x, y) {
                (Some(Rc::clone(entity)), EntityState::is_pc(entity))
            } else {
                (None, false)
            };
        Rc::new(ActionMenu {
            area_state: area_state,
            area_pos: Point::new(x, y),
            hovered_entity,
            is_hover_pc,
        })
    }

    pub fn transition_callback(&self, transition: &Transition) -> Option<Callback<Button>> {
        let area_id = transition.to_area.clone();
        let x = transition.to.x;
        let y = transition.to.y;

        Some(Callback::new(Rc::new( move |_kind, widget| {
            GameState::transition(&area_id, x, y);
            Widget::mark_removal_up_tree(&widget, 2);
            let root = Widget::get_root(&widget);
            root.borrow_mut().invalidate_children();
        })))
    }

    pub fn attack_callback(&self) -> Box<Fn()> {
        Box::new( || { })
    }

    pub fn is_move_valid(&self) -> bool {
        let pc = GameState::pc();
        let size = pc.borrow().size();
        let ok = self.area_state.borrow().is_passable(&pc.borrow(),
            self.area_pos.x - size / 2,
            self.area_pos.y - size / 2);
        ok
    }

    pub fn move_callback(&self) -> Box<Fn()> {
        let pc = GameState::pc();
        let size = pc.borrow().size();
        let x = self.area_pos.x - size / 2;
        let y = self.area_pos.y - size / 2;
        Box::new(move || {
            GameState::pc_move_to(x, y);
        })
    }

    fn callback_with_removal(f: Box<Fn()>) -> Option<Callback<Button>> {
        Some(Callback::new(Rc::new(move |_kind, widget| {
            f();
            Widget::mark_removal_up_tree(&widget, 2);
        })))
    }

    pub fn fire_default_callback(&self) {
        (self.move_callback())();
    }

    pub fn is_default_callback_valid(&self) -> bool {
        self.is_move_valid()
    }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str {
        "action_menu"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry> = Vec::new();
        if self.is_move_valid() {
            entries.push(list_box::Entry::new(
                    "Move", ActionMenu::callback_with_removal(self.move_callback())));

        }

        if let Some(ref _entity) = self.hovered_entity {
            if !self.is_hover_pc {
                entries.push(list_box::Entry::new(
                        "Attack", ActionMenu::callback_with_removal(self.attack_callback())));
            }
        }

        if let Some(ref transition) = self.area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
            entries.push(list_box::Entry::new(
                    "Transition", self.transition_callback(transition)));
        }

        if entries.is_empty() {
            entries.push(list_box::Entry::new(
                    "None", ActionMenu::callback_with_removal(Box::new(|| { }))));
        }
        let actions = Widget::with_theme(ListBox::new(entries), "actions");

        vec![title, actions]
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }

        true
    }
}
