mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod area_view;
pub use self::area_view::AreaView;

use std::rc::Rc;
use std::cell::RefCell;

use io::{event, Event, InputAction, TextRenderer};
use resource::Point;
use ui::{Button, Cursor, EmptyWidget, Label, Widget, WidgetKind, Window};
use state::{AreaState, GameState};

pub struct RootView {
    area_state: Rc<RefCell<AreaState>>,
}

impl RootView {
    pub fn new() -> Rc<RootView> {
        Rc::new(RootView {
            area_state: GameState::area_state(),
        })
    }

    pub fn cursor_move_by(&self, root: &Rc<RefCell<Widget>>,
                          x: i32, y: i32) {
        trace!("Emulating cursor move by {}, {} as mouse event", x, y);
        if !Cursor::move_by(x, y) {
            return;
        }

        let event = Event::new(event::Kind::MouseMove { change: Point::new(x, y) },
            Cursor::get_x(), Cursor::get_y());
        Widget::dispatch_event(&root, event);
    }

    pub fn cursor_click(&self, root: &Rc<RefCell<Widget>>) {
        let (x, y) = Cursor::get_position();

        trace!("Emulating cursor click event at {},{} as mouse event", x, y);
        let event = Event::new(event::Kind::MouseClick(event::ClickKind::Left), x, y);
        Widget::dispatch_event(&root, event);
    }
}

impl WidgetKind for RootView {
    fn get_name(&self) -> &str {
        "root"
    }

    fn after_draw_text_mode(&self, renderer: &mut TextRenderer,
                            _widget: &Widget, millis: u32) {
        Cursor::draw_text_mode(renderer, millis);
    }

    fn before_dispatch_event(&self, root: &Rc<RefCell<Widget>>, event: Event) -> bool {
        use io::InputAction::*;
        match event.kind {
            event::Kind::KeyPress(input_action) => {
                match input_action {
                    MoveCursorUp => self.cursor_move_by(root, 0, -1),
                    MoveCursorDown => self.cursor_move_by(root, 0, 1),
                    MoveCursorLeft => self.cursor_move_by(root, -1, 0),
                    MoveCursorRight => self.cursor_move_by(root, 1, 0),
                    ClickCursor => self.cursor_click(root),
                    _ => return false
                }

                true
            },
            _ => false
        }
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>,
                    key: InputAction, _mouse_pos: Point) -> bool {
        use io::InputAction::*;
        match key {
            Exit => {
                    let exit_window = Widget::with_theme(Window::new(), "exit_window");
                    exit_window.borrow_mut().state.set_modal(true);
                    Widget::add_child_to(&widget, exit_window);
                    true
            },
            ToggleInventory => {
                let window = Widget::get_child_with_name(widget,
                                self::inventory_window::NAME);
                match window {
                    None => {
                        let window = Widget::with_defaults(
                            InventoryWindow::new(&GameState::pc()));
                        Widget::add_child_to(&widget, window);
                    },
                    Some(window) => {
                        window.borrow_mut().mark_for_removal();
                    }
                }
                true
            },
            _ => {
                false
            }
        }
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to root widget.");

        let mouse_over = Widget::with_theme(Label::empty(), "mouse_over");

        let area_widget = Widget::with_defaults(
            AreaView::new(&self.area_state, Rc::clone(&mouse_over)));

        let right_pane = Widget::with_theme(EmptyWidget::new(), "right_pane");
        {
            let button = Widget::with_theme(
                Button::with_callback(Rc::new(|_k, _w| info!("Hello world"))),
                "test_button");

            let area_title = Widget::with_theme(
                Label::new(&self.area_state.borrow().area.name), "title");
            Widget::add_child_to(&right_pane, mouse_over);
            Widget::add_child_to(&right_pane, button);
            Widget::add_child_to(&right_pane, area_title);
        }

        vec![area_widget, right_pane]
    }
}
