use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use grt::ui::{Cursor, Label, WidgetKind, Widget};
use grt::io::{DrawList, InputAction, TextRenderer};
use grt::io::event::ClickKind;

use view::ActionMenu;
use state::{AreaState, GameState};

pub struct AreaView {
    area_state: Rc<RefCell<AreaState>>,
    mouse_over: Rc<RefCell<Widget>>,
}

impl AreaView {
    pub fn new(area_state: &Rc<RefCell<AreaState>>,
               mouse_over: Rc<RefCell<Widget>>) -> Rc<AreaView> {
        Rc::new(AreaView {
            area_state: Rc::clone(area_state),
            mouse_over: mouse_over,
        })
    }

}

impl WidgetKind for AreaView {
    fn get_name(&self) -> &str {
        "area"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let width = self.area_state.borrow().area.width;
        let height = self.area_state.borrow().area.height;
        widget.borrow_mut().state.set_max_scroll_pos(width, height);
        self.mouse_over.borrow_mut().state.add_text_param("");
        self.mouse_over.borrow_mut().state.add_text_param("");

        Vec::with_capacity(0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      widget: &Widget, _millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.inner_size;

        let state = self.area_state.borrow();
        let ref area = state.area;

        let max_x = cmp::min(s.width, area.width - widget.state.scroll_pos.x);
        let max_y = cmp::min(s.height, area.height - widget.state.scroll_pos.y);

        renderer.set_cursor_pos(0, 0);

        for y in 0..max_y {
            renderer.set_cursor_pos(p.x, p.y + y);
            for x in 0..max_x {
                renderer.render_char(state.get_display(x + widget.state.scroll_pos.x,
                                                       y + widget.state.scroll_pos.y));
            }
        }
    }

    fn get_draw_lists(&self, widget: &Widget, _millis: u32) -> Vec<DrawList> {
        let p = widget.state.inner_position;
        let s = widget.state.inner_size;

        let state = self.area_state.borrow();
        let ref area = state.area;

        let max_x = cmp::min(s.width, area.width - widget.state.scroll_pos.x);
        let max_y = cmp::min(s.height, area.height - widget.state.scroll_pos.y);

        let mut draw_list = DrawList::empty_sprite();
        for y in 0..max_y {
            for x in 0..max_x {
                let area_x = x + widget.state.scroll_pos.x;
                let area_y = y + widget.state.scroll_pos.y;

                let tile = match area.terrain.tile_at(area_x, area_y) {
                    &None => continue,
                    &Some(ref tile) => tile,
                };

                draw_list.append(&mut DrawList::from_sprite(&tile.image_display,
                                                            p.x + x, p.y + y,
                                                            tile.width, tile.height));
            }
        }

        for entity in state.entities.iter() {
            let entity = entity.borrow();
            draw_list.append(&mut DrawList::from_sprite(&entity.actor.actor.image_display,
                                                        entity.location.x, entity.location.y,
                                                        entity.size(), entity.size()));
        }

        let pc = GameState::pc();
        let size = pc.borrow().size();
        let cursors = DrawList::from_sprite(&pc.borrow().size.cursor_sprite
                                            , Cursor::get_x() - size / 2, Cursor::get_y() - size / 2
                                            , size, size);

        vec![draw_list, cursors]
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {

        use grt::io::InputAction::*;
        match key {
           ScrollUp => widget.borrow_mut().state.scroll(0, -1),
           ScrollDown => widget.borrow_mut().state.scroll(0, 1),
           ScrollLeft => widget.borrow_mut().state.scroll(-1, 0),
           ScrollRight => widget.borrow_mut().state.scroll(1, 0),
           _ => return false,
        };
        true
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let pos = widget.borrow().state.position;
        let x = Cursor::get_x() - pos.x + widget.borrow().state.scroll_pos.x;
        let y = Cursor::get_y() - pos.y + widget.borrow().state.scroll_pos.y;
        if x < 0 || y < 0 { return true; }

        let action_menu = ActionMenu::new(Rc::clone(&self.area_state), x, y);
        if kind == ClickKind::Left {
            action_menu.fire_default_callback();
        } else if kind == ClickKind::Right {
            Widget::add_child_to(widget, Widget::with_defaults(action_menu));
        }

        true
    }

    fn on_mouse_move(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        let pos = widget.borrow().state.position;
        let area_x = Cursor::get_x() - pos.x + widget.borrow().state.scroll_pos.x;
        let area_y = Cursor::get_y() - pos.y + widget.borrow().state.scroll_pos.y;

        {
            let ref mut state = self.mouse_over.borrow_mut().state;
            state.clear_text_params();
            state.add_text_param(&format!("{}", area_x));
            state.add_text_param(&format!("{}", area_y));
        }
        self.mouse_over.borrow_mut().invalidate_layout();

        if let Some(entity) = self.area_state.borrow().get_entity_at(area_x, area_y) {
            let index = entity.borrow().index;
            let pc = GameState::pc();
            if index != pc.borrow().index {
                Widget::set_mouse_over(widget, Label::new(&entity.borrow().actor.actor.id));
            }
        }
        true
    }

    fn on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.mouse_over.borrow_mut().state.clear_text_params();
        true
    }
}
