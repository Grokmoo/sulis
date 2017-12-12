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
    setup_widgets(&mut root, area_state, resource_set);

    root
}
fn setup_widgets<'a>(root: &mut Widget<'a>,
    area_state: Rc<RefCell<AreaState<'a>>>, resource_set: &ResourceSet) {
    let right_pane_width = 20;

    let area_width = cmp::min(area_state.borrow().area.width,
        root.state.size.width - right_pane_width);
    let area_height = cmp::min(area_state.borrow().area.height,
        root.state.size.height - 1);
    let area_title = area_state.borrow().area.name.clone();

    root.add_child(Widget::with_size(
            Label::new(&area_title),
            Size::new(area_width, 1),
            ));

    let mouse_over_label = Label::new_empty();
    let mouse_over_label2 = Rc::clone(&mouse_over_label);
    let mouse_over = Widget::with_position(
        mouse_over_label,
        Size::new(right_pane_width, 1),
        Point::new(area_width + 1, 0),
        );

    // let mouse_over_ref = WidgetRef::new(mouse_over_label2, Rc::clone(&mouse_over));
    // root.add_child(WidgetState::with_border(
    //         AreaWidget::new(area_state, mouse_over_ref),
    //         Size::new(area_width, area_height),
    //         Point::new(0, 1),
    //         Border::as_uniform(0),
    //         ));

    let mut button = Widget::with_position(
            Button::new("Test"),
            Size::new(right_pane_width, 3),
            Point::new(area_width + 1, 3),
            );
    button.state.set_background(resource_set.get_image("background"));
    root.add_child(button);

    root.add_child(mouse_over);
}
