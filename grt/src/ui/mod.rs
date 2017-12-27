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

pub mod animation_state;
pub use self::animation_state::AnimationState;

mod label;
pub use self::label::Label;

mod button;
pub use self::button::Button;

pub mod list_box;
pub use self::list_box::ListBox;

mod confirmation_window;
pub use self::confirmation_window::ConfirmationWindow;

mod cursor;
pub use self::cursor::Cursor;

mod callback;
pub use self::callback::Callback;

use std::rc::Rc;
use std::cell::RefCell;

use config::CONFIG;
use resource::ResourceSet;

pub fn create_ui_tree(kind: Rc<WidgetKind>) -> Rc<RefCell<Widget>> {

    debug!("Creating UI tree.");
    let root = Widget::with_defaults(kind);
    root.borrow_mut().state.set_size(Size::new(CONFIG.display.width,
                                               CONFIG.display.height));
    root.borrow_mut().theme = Some(ResourceSet::get_theme());

    root
}
