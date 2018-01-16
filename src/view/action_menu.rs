use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{Callback, Cursor, Label, list_box, ListBox, Widget, WidgetKind};
use grt::io::event::ClickKind;
use grt::util::Point;

use state::{AreaState, ChangeListener, GameState, EntityState};

const NAME: &'static str = "action_menu";

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

    pub fn is_transition_valid(&self) -> bool {
        if let Some(transition) = self.area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                let pc = GameState::pc();
                return pc.borrow().dist_to_transition(transition) < 2.5;
            }
        false
    }

    pub fn transition_callback(&self) -> Option<Callback> {
        let (area_id, x, y) = match self.area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                None => return None,
                Some(transition) => (transition.to_area.clone(), transition.to.x, transition.to.y)
            };

        Some(Callback::new(Rc::new( move |widget| {
            trace!("Firing transition callback.");
            GameState::transition(&area_id, x, y);
            Widget::mark_removal_up_tree(&widget, 2);
            let root = Widget::get_root(&widget);
            root.borrow_mut().invalidate_children();
        })))
    }

    pub fn is_attack_valid(&self) -> bool {
        if self.is_hover_pc { return false; }
        let pc = GameState::pc();

        match self.hovered_entity {
            None => false,
            Some(ref entity) => pc.borrow().actor.can_attack(entity),
        }
    }

    pub fn attack_callback(&self) -> Box<Fn()> {
        if let Some(ref entity) = self.hovered_entity {
            let entity_ref = Rc::clone(entity);
            Box::new(move || {
                trace!("Firing attack callback.");
                let pc = GameState::pc();
                pc.borrow_mut().actor.attack(&entity_ref);
            })
        } else {
            Box::new(|| { })
        }
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
            trace!("Firing move callback.");
            GameState::pc_move_to(x, y);
        })
    }

    fn callback_with_removal(f: Box<Fn()>) -> Option<Callback> {
        Some(Callback::new(Rc::new(move |widget| {
            f();
            Widget::mark_removal_up_tree(&widget, 2);
        })))
    }

    pub fn fire_default_callback(&self) {
        if self.is_attack_valid() {
            (self.attack_callback())();
        } else if self.is_move_valid() {
            (self.move_callback())();
        }
    }

    pub fn is_default_callback_valid(&self) -> bool {
        self.is_move_valid()
    }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_remove(&self) {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.remove(NAME);
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.add(ChangeListener::remove_widget(NAME, widget));

        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        if self.is_move_valid() {
            entries.push(list_box::Entry::new(
                    "Move".to_string(), ActionMenu::callback_with_removal(self.move_callback())));

        }

        if self.is_attack_valid() {
            entries.push(list_box::Entry::new(
                    "Attack".to_string(), ActionMenu::callback_with_removal(self.attack_callback())));
        }

        if self.is_transition_valid() {
            entries.push(list_box::Entry::new(
                    "Transition".to_string(), self.transition_callback()));
        }

        if entries.is_empty() {
            entries.push(list_box::Entry::new(
                    "None".to_string(), ActionMenu::callback_with_removal(Box::new(|| { }))));
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
