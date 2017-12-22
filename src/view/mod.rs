mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod area_view;
pub use self::area_view::AreaView;

use std::rc::Rc;
use std::cell::RefCell;

use io::InputAction;
use resource::Point;
use ui::{Button, EmptyWidget, Label, Widget, WidgetKind, Window};
use state::{AreaState, GameState};

pub struct RootView<'a> {
    area_state: Rc<RefCell<AreaState<'a>>>,
}

impl<'a> RootView<'a> {
    pub fn new(area_state: Rc<RefCell<AreaState<'a>>>) -> Rc<RootView<'a>> {
        Rc::new(RootView {
            area_state,
        })
    }
}

impl<'a> WidgetKind<'a> for RootView<'a> {
    fn get_name(&self) -> &str {
        "root"
    }

    fn on_key_press(&self, state: &mut GameState, widget: &Rc<RefCell<Widget<'a>>>,
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
                            InventoryWindow::new(&state.pc));
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

    fn on_add(&self, _widget: &Rc<RefCell<Widget<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {
        debug!("Adding to root widget.");

        let mouse_over = Widget::with_theme(Label::empty(), "mouse_over");

        let area_widget = Widget::with_defaults(
            AreaView::new(&self.area_state, Rc::clone(&mouse_over)));

        let right_pane = Widget::with_theme(EmptyWidget::new(), "right_pane");
        {
            let button = Widget::with_theme(
                Button::with_callback(Box::new(|_w, _s| info!("Hello world"))),
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
