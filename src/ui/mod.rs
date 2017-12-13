pub mod theme;

mod widget;
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

use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use state::AreaState;
use config::Config;
use resource::{Point, ResourceSet};

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
    config: &Config, resource_set: &ResourceSet) -> Widget<'a> {

    debug!("Creating UI tree.");
    let mut root = Widget::with_size(Rc::new(EmptyWidget {}),
        Size::new(config.display.width, config.display.height));
    root.state.set_border(Border::as_uniform(1));
    root.state.set_background(resource_set.get_image("background"));
    setup_widgets(&mut root, area_state, resource_set);

    root
}
fn setup_widgets<'a>(root: &mut Widget<'a>,
    area_state: Rc<RefCell<AreaState<'a>>>, resource_set: &ResourceSet) {
    let right_pane_width = 20;

    let area_width = cmp::min(area_state.borrow().area.width,
        root.state.inner_size.width - right_pane_width);
    let area_height = cmp::min(area_state.borrow().area.height,
        root.state.inner_size.height - 1);

    let mouse_over = Rc::new(RefCell::new(Widget::with_size(
        Label::new(),
        Size::new(right_pane_width, 1),
        )));

    let area_widget = Widget::with_border(
            AreaWidget::new(&area_state, Rc::clone(&mouse_over)),
            Size::new(area_width, area_height),
            Point::new(root.state.inner_position.x, root.state.inner_position.y + 1),
            Border::as_uniform(0));
    let area_right = area_widget.state.get_right();
    root.add_child(area_widget);

    mouse_over.borrow_mut().state
        .set_position(area_right + 1, root.state.inner_position.y);

    let mut button = Widget::with_position(
            Button::new(Box::new(|_w, _s| trace!("Hello world"))),
            Size::new(right_pane_width - 1, 3),
            Point::new(area_right + 1, root.state.inner_position.y + 2));
    button.state.set_text("Test");
    button.state.set_background(resource_set.get_image("background"));
    root.add_child(button);

    let mut area_title = Widget::with_position(
        Label::new(),
        Size::new(area_width, 1),
        root.state.inner_position);
    area_title.state.set_text(&area_state.borrow().area.name);
    root.add_child(area_title);

    root.add_child_rc(mouse_over);
}
