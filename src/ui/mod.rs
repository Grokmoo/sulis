pub mod theme;

pub mod widget;
pub use self::widget::Widget;

mod widget_state;
pub use self::widget_state::WidgetState;

mod widget_kind;
pub use self::widget_kind::WidgetKind;
pub use self::widget_kind::EmptyWidget;

mod border;
pub use self::border::Border;

mod size;
pub use self::size::Size;

mod animation_state;
pub use self::animation_state::AnimationState;

mod area_widget;
pub use self::area_widget::AreaWidget;

mod label;
pub use self::label::Label;

mod button;
pub use self::button::Button;

mod window;
pub use self::window::Window;

use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use state::AreaState;
use config::Config;
use resource::Point;

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
    config: &Config) -> Rc<RefCell<Widget<'a>>> {

    debug!("Creating UI tree.");
    let mut root = Widget::with_border(
        Rc::new(EmptyWidget {}),
        Size::new(config.display.width, config.display.height),
        Point::as_zero(),
        Border::as_uniform(1));
    Widget::set_background(&mut root, "background");

    let widgets_to_add = setup_widgets(Rc::clone(&root), area_state);
    Widget::add_children_to(&root, widgets_to_add);

    root
}
fn setup_widgets<'a>(root: Rc<RefCell<Widget<'a>>>,
    area_state: Rc<RefCell<AreaState<'a>>>) -> Vec<Rc<RefCell<Widget<'a>>>> {

    let right_pane_width = 20;

    let ref state = root.borrow().state;
    let area_width = cmp::min(area_state.borrow().area.width,
        state.inner_size.width - right_pane_width);
    let area_height = cmp::min(area_state.borrow().area.height,
        state.inner_size.height - 1);
    let right_pane_x = state.inner_right() - right_pane_width;

    let mouse_over = Widget::with_size(
        Label::new(),
        Size::new(right_pane_width, 1),
        );

    let area_widget = Widget::with_border(
            AreaWidget::new(&area_state, Rc::clone(&mouse_over)),
            Size::new(area_width, area_height),
            Point::new(state.inner_position.x, state.inner_position.y + 1),
            Border::as_uniform(0));

    mouse_over.borrow_mut().state
        .set_position(right_pane_x, state.inner_position.y);

    let mut button = Widget::with_position(
            Button::new(Box::new(|_w, _s| trace!("Hello world"))),
            Size::new(right_pane_width - 1, 3),
            Point::new(right_pane_x, state.inner_position.y + 2));
    Widget::set_text(&mut button, "Test");
    Widget::set_background(&mut button, "background");

    let mut area_title = Widget::with_position(
        Label::new(),
        Size::new(area_width, 1),
        state.inner_position);
    Widget::set_text(&mut area_title, &area_state.borrow().area.name);

    vec![area_widget, button, area_title, mouse_over]
}
