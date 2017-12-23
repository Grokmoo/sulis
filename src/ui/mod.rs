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

mod window;
pub use self::window::Window;

use std::rc::Rc;
use std::cell::RefCell;

use state::{AreaState, GameState};
use config::Config;
use resource::ResourceSet;
use view::RootView;

pub type Callback<T> = Rc<Fn(&T, &Rc<RefCell<Widget>>, &mut GameState)>;

pub fn create_ui_tree(area_state: Rc<RefCell<AreaState>>,
    config: &Config) -> Rc<RefCell<Widget>> {

    debug!("Creating UI tree.");
    let root = Widget::with_defaults(RootView::new(area_state));
    root.borrow_mut().state.set_size(Size::new(config.display.width,
                                               config.display.height));
    root.borrow_mut().theme = Some(ResourceSet::get_theme());

    root
}
