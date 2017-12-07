mod widget_base;
pub use self::widget_base::WidgetBase;

mod widget;
pub use self::widget::Widget;
pub use self::widget::EmptyWidget;

mod widget_ref;
pub use self::widget_ref::WidgetRef;

mod area_widget;
pub use self::area_widget::AreaWidget;

mod label;
pub use self::label::Label;

use std::rc::Rc;
use std::cell::{RefCell, RefMut};

use state::AreaState;
use io::IO;

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
                      display: &Box<IO + 'a>) -> Rc<RefCell<WidgetBase<'a>>> {

    let root = WidgetBase::default(Rc::new(RefCell::new(EmptyWidget {})));

    setup_widgets(root.borrow_mut(), area_state);

    let (width, height) = display.get_display_size();

    root.borrow_mut().set_size(width, height);

    root
}

fn setup_widgets<'a>(ref mut root: RefMut<WidgetBase<'a>>,
                     area_state: Rc<RefCell<AreaState<'a>>>) {
    let area_width = area_state.borrow().area.width;
    let area_height = area_state.borrow().area.height;
    let area_title = area_state.borrow().area.name.clone();

    root.add_child(WidgetBase::new(
        Label::new(&area_title),
        0, 0,
        area_width as i32, 1,
    ));

    let mouse_over_label = Label::new_empty();
    let mouse_over_label2 = Rc::clone(&mouse_over_label);
    let mouse_over = WidgetBase::new(
        mouse_over_label,
        0, 0,
        10, 1,
    );

    let mouse_over_ref = WidgetRef::new(mouse_over_label2, Rc::clone(&mouse_over));
    root.add_child(WidgetBase::new(
        AreaWidget::new(area_state, mouse_over_ref),
        1, 1,
        area_width as i32, area_height as i32,
    ));

    root.add_child(mouse_over);
}
