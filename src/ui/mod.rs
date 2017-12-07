use std::rc::Rc;
use std::cell::RefCell;

use state::{AreaState, GameState};
use io::{IO, TextRenderer};

pub trait Widget {
    fn draw_text_mode(&self, _renderer: &mut TextRenderer, _x: usize, _y: usize) { }

    fn on_click(&self, _parent: &WidgetState, _state: &mut GameState,
                _x: i32, _y: i32) -> bool {
        false
    }
}

pub struct WidgetState<'a> {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub children: Vec<WidgetState<'a>>,
    drawable: Box<Widget + 'a>,
}

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
                      display: &Box<IO + 'a>) -> WidgetState<'a> {
    let mut root = WidgetState::new(Box::new(EmptyWidget {}));

    let area_width = area_state.borrow().area.width;
    let area_height = area_state.borrow().area.height;
    let area_title = area_state.borrow().area.name.clone();

    let (width, height) = display.get_display_size();

    root.set_size(width, height);
    root.set_position(0, 0);

    let mut title = WidgetState::new(Box::new(Label { text: area_title }));
    title.set_position(0, 0);
    title.set_size(area_width as i32, 1);
    root.add_child(title);

    let mut area = WidgetState::new(
        Box::new(AreaWidget::new(area_state))
        );
    area.set_position(1, 1);
    area.set_size(area_width as i32, area_height as i32);
    root.add_child(area);

    root
}

impl<'a> WidgetState<'a> {
    fn new(drawable: Box<Widget + 'a>) -> WidgetState<'a> {
        WidgetState {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            children: Vec::new(),
            drawable,
        }
    }

    pub fn dispatch_click(&self, state: &mut GameState, x: i32, y: i32) -> bool {
        for child in self.children.iter() {
            if child.is_in_bounds(x, y) {
                if child.dispatch_click(state, x, y) {
                    return true;
                }
            }
        }

        self.drawable.on_click(&self, state, x, y)
    }

    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y &&
            x <= self.x + self.width && y <= self.y + self.height
    }

    pub fn add_child(&mut self, widget: WidgetState<'a>) {
        self.children.push(widget);
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
        self.drawable.draw_text_mode(renderer, self.x as usize, self.y as usize);

        for child in self.children.iter() {
            child.draw_text_mode(renderer);
        }
    }
}

pub struct EmptyWidget { }
impl Widget for EmptyWidget { }

pub struct AreaWidget<'a> {
    area_state: Rc<RefCell<AreaState<'a>>>,
}

impl<'a> AreaWidget<'a> {
    pub fn new(area_state: Rc<RefCell<AreaState<'a>>>) -> AreaWidget<'a> {
        AreaWidget {
            area_state
        }
    }
}

impl<'a> Widget for AreaWidget<'a> {
    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      x_start: usize, y_start: usize) {
        let state = self.area_state.borrow();
        let ref area = state.area;
        for y in 0..area.height {
            renderer.set_cursor_pos(x_start as u32, (y + y_start) as u32);
            for x in 0..area.width {
                renderer.render_char(state.get_display(x, y));
            }
        }
    }

    fn on_click(&self, parent: &WidgetState, state: &mut GameState,
                x: i32, y: i32) -> bool {
        let size = state.pc().size() as i32;
        let x = (x - parent.x) - size / 2;
        let y = (y - parent.y) - size / 2;
        if x >= 0 && y >= 0 {
            state.pc_move_to(x as usize, y as usize);
        }

        true // consume the event
    }
}

pub struct Label {
    text: String,
}

impl Widget for Label {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, x_start: usize, y_start: usize) {
        renderer.set_cursor_pos(x_start as u32, y_start as u32);
        renderer.render_string(&self.text);
    }
}
