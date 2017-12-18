pub mod theme;
pub use self::theme::Theme;

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

use state::AreaState;
use config::Config;
use resource::ResourceSet;

pub fn create_ui_tree<'a>(area_state: Rc<RefCell<AreaState<'a>>>,
    config: &Config) -> Rc<RefCell<Widget<'a>>> {

    debug!("Creating UI tree.");
    let root = Widget::with_defaults(EmptyWidget::new());
    root.borrow_mut().state.set_size(Size::new(config.display.width,
                                               config.display.height));
    root.borrow_mut().theme = Some(ResourceSet::get_theme());

    let widgets_to_add = setup_widgets(area_state);
    Widget::add_children_to(&root, widgets_to_add);

    root
}
fn setup_widgets<'a>(area_state: Rc<RefCell<AreaState<'a>>>) ->
    Vec<Rc<RefCell<Widget<'a>>>> {


    let mouse_over = Widget::with_theme(Label::empty(), "mouse_over");

    let area_widget = Widget::with_defaults(
        AreaWidget::new(&area_state, Rc::clone(&mouse_over)));

    let right_pane = Widget::with_theme(EmptyWidget::new(), "right_pane");
    {
        let button = Widget::with_theme(
            Button::new(Box::new(|_w, _s| info!("Hello world"))),
            "test_button");

        let area_title = Widget::with_theme(
            Label::new(&area_state.borrow().area.name), "title");
        Widget::add_child_to(&right_pane, mouse_over);
        Widget::add_child_to(&right_pane, button);
        Widget::add_child_to(&right_pane, area_title);
    }

    vec![area_widget, right_pane]
}
