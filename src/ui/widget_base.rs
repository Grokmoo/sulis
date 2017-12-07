use std::rc::Rc;
use std::cell::RefCell;

use ui::Widget;
use state::GameState;
use io::{MouseEvent, TextRenderer};

//// The base widget holder class.  Contains the common implementation across all
//// widgets, and holds an instance of 'Widget' which contains the specific behavior.
pub struct WidgetBase<'a> {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub children: Vec<Rc<RefCell<WidgetBase<'a>>>>,
    drawable: Rc<RefCell<Widget + 'a>>,
}

impl<'a> WidgetBase<'a> {
    pub fn default(drawable: Rc<RefCell<Widget + 'a>>) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            drawable,
            x: 0, y: 0,
            width: 0, height: 0,
            children: Vec::new(),
        }))
    }

    pub fn new(drawable: Rc<RefCell<Widget + 'a>>,
               pos_x: i32, pos_y: i32,
               width: i32, height: i32) -> Rc<RefCell<WidgetBase<'a>>> {
        Rc::new(RefCell::new(WidgetBase {
            x: pos_x,
            y: pos_y,
            width,
            height,
            children: Vec::new(),
            drawable,
        }))
    }

    pub fn dispatch_event(&self, state: &mut GameState, event: MouseEvent) -> bool {
        for child in self.children.iter() {
            let child = child.borrow();
            if child.is_in_bounds(event.x as i32, event.y as i32) {
                if child.dispatch_event(state, event) {
                    return true;
                }
            }
        }

        let drawable = self.drawable.borrow_mut();
        use io::mouse_event::Kind::*;
        match event.kind {
            LeftClick => drawable.on_left_click(&self, state, event.x, event.y),
            MiddleClick => drawable.on_middle_click(&self, state, event.x, event.y),
            RightClick => drawable.on_right_click(&self, state, event.x, event.y),
            Move(_, _) => drawable.on_mouse_moved(&self, state, event.x, event.y),
            _ => false,
        }
    }

    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y &&
            x < self.x + self.width && y < self.y + self.height
    }

    pub fn add_child(&mut self, widget: Rc<RefCell<WidgetBase<'a>>>) {
        self.children.push(widget);
    }

    pub fn set_position_centered(&mut self, x: i32, y: i32) {
        self.x = x - (self.width - 1) / 2;
        self.y = y - (self.height - 1) / 2;
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn draw_text_mode(&self, renderer: &mut TextRenderer) {
        self.drawable.borrow_mut().draw_text_mode(renderer, &self);

        for child in self.children.iter() {
            let child = child.borrow();
            child.draw_text_mode(renderer);
        }
    }
}
