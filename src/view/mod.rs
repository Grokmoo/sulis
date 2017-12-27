mod inventory_window;
pub use self::inventory_window::InventoryWindow;

mod area_view;
pub use self::area_view::AreaView;

use std::rc::Rc;
use std::cell::RefCell;

use grt::io::InputAction;
use grt::util::Point;
use grt::ui::{Button, ConfirmationWindow, EmptyWidget, Label, Widget, WidgetKind};
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
}

impl WidgetKind for RootView {
    fn get_name(&self) -> &str {
        "root"
    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>,
                    key: InputAction, _mouse_pos: Point) -> bool {
        use grt::io::InputAction::*;
        match key {
            Exit => {
                    let exit_window = Widget::with_theme(
                        ConfirmationWindow::new(Rc::new(|_k, _w| { GameState::set_exit(); })),
                        "exit_confirmation_window");
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
