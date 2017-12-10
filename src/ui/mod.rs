pub mod theme;

mod widget_base;
pub use self::widget_base::WidgetBase;

mod widget;
pub use self::widget::Widget;
pub use self::widget::EmptyWidget;

mod widget_ref;
pub use self::widget_ref::WidgetRef;

mod border;
pub use self::border::Border;

mod size;
pub use self::size::Size;

mod area_widget;
pub use self::area_widget::AreaWidget;

mod label;
pub use self::label::Label;

mod button;
pub use self::button::Button;

use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::cmp;

use state::AreaState;
use config::Config;
use resource::{Point, ResourceSet};

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
    config: &Config, resource_set: &ResourceSet) -> Rc<RefCell<WidgetBase<'a>>> {

    debug!("Creating UI tree.");
    let root = WidgetBase::with_size(Rc::new(RefCell::new(EmptyWidget {})),
        Size::new(config.display.width, config.display.height));
    setup_widgets(root.borrow_mut(), area_state, resource_set);

    root
}
fn setup_widgets<'a>(ref mut root: RefMut<WidgetBase<'a>>,
    area_state: Rc<RefCell<AreaState<'a>>>, resource_set: &ResourceSet) {
    let right_pane_width = 20;

    let area_width = cmp::min(area_state.borrow().area.width,
        root.size.width - right_pane_width);
    let area_height = cmp::min(area_state.borrow().area.height,
        root.size.height - 1);
    let area_title = area_state.borrow().area.name.clone();

    root.add_child(WidgetBase::with_size(
            Label::new(&area_title),
            Size::new(area_width, 1),
            ));

    let mouse_over_label = Label::new_empty();
    let mouse_over_label2 = Rc::clone(&mouse_over_label);
    let mouse_over = WidgetBase::with_position(
        mouse_over_label,
        Size::new(right_pane_width, 1),
        Point::new(area_width + 1, 0),
        );

    let mouse_over_ref = WidgetRef::new(mouse_over_label2, Rc::clone(&mouse_over));
    root.add_child(WidgetBase::with_border(
            AreaWidget::new(area_state, mouse_over_ref),
            Size::new(area_width, area_height),
            Point::new(0, 1),
            Border::as_uniform(0),
            ));

    let button = WidgetBase::with_position(
            Button::new("Test"),
            Size::new(right_pane_width, 3),
            Point::new(area_width + 1, 3),
            );
    button.borrow_mut().set_background(resource_set.get_image("bg"));
    root.add_child(button);

    root.add_child(mouse_over);
}
