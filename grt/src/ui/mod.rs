pub mod theme;
pub use self::theme::Theme;

pub mod markup_renderer;
pub use self::markup_renderer::MarkupRenderer;

pub mod color;
pub use self::color::Color;

pub mod widget;
pub use self::widget::Widget;

mod widget_state;
pub use self::widget_state::WidgetState;

mod widget_kind;
pub use self::widget_kind::WidgetKind;
pub use self::widget_kind::EmptyWidget;

mod border;
pub use self::border::Border;

pub mod animation_state;
pub use self::animation_state::AnimationState;

mod label;
pub use self::label::Label;

mod button;
pub use self::button::Button;

pub mod list_box;
pub use self::list_box::ListBox;

pub mod text_area;
pub use self::text_area::TextArea;

mod confirmation_window;
pub use self::confirmation_window::ConfirmationWindow;

mod cursor;
pub use self::cursor::Cursor;

mod callback;
pub use self::callback::Callback;

mod font_renderer;
pub use self::font_renderer::FontRenderer;
pub use self::font_renderer::LineRenderer;

use std::rc::Rc;
use std::cell::RefCell;

use config::CONFIG;
use resource::ResourceSet;
use util::Size;

pub fn create_ui_tree(kind: Rc<WidgetKind>) -> Rc<RefCell<Widget>> {

    debug!("Creating UI tree.");
    let root = Widget::with_defaults(kind);
    root.borrow_mut().state.set_size(Size::new(CONFIG.display.width,
                                               CONFIG.display.height));
    root.borrow_mut().theme = Some(ResourceSet::get_theme());

    root
}
