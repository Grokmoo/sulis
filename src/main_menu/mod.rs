use std::rc::Rc;
use std::cell::RefCell;

use grt::io::MainLoopUpdater;
use grt::ui::{animation_state, Button, Callback, Label, list_box, ListBox, WidgetKind, Widget};

pub struct MainMenuLoopUpdater {
    main_menu_view: Rc<MainMenuView>,
}

impl MainMenuLoopUpdater {
    pub fn new(main_menu_view: &Rc<MainMenuView>) -> MainMenuLoopUpdater {
        MainMenuLoopUpdater {
            main_menu_view: Rc::clone(main_menu_view),
        }
    }
}

impl MainLoopUpdater for MainMenuLoopUpdater {
    fn update(&self) { }

    fn is_exit(&self) -> bool {
        self.main_menu_view.is_exit()
    }
}

pub struct MainMenuView {
    modules: Vec<String>,
}

impl MainMenuView {
    pub fn new(modules: Vec<String>) -> Rc<MainMenuView> {
        Rc::new(MainMenuView {
            modules,
        })
    }

    pub fn is_exit(&self) -> bool {
        EXIT.with(|exit| *exit.borrow())
    }
}

thread_local! {
    static EXIT: RefCell<bool> = RefCell::new(false);
}

impl WidgetKind for MainMenuView {
    fn get_name(&self) -> &str {
        "root"
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        debug!("Adding to main menu widget");

        let title = Widget::with_theme(Label::empty(), "title");
        let modules_title = Widget::with_theme(Label::empty(), "modules_title");

        let mut entries: Vec<list_box::Entry> = Vec::new();
        let cb: Callback<Button> = Callback::new(Rc::new( |_, widget| {
            widget.borrow_mut().state.animation_state.toggle(animation_state::Kind::Active);
        }));
        for module in self.modules.iter() {
            let entry = list_box::Entry::new(&module, Some(cb.clone()));
            entries.push(entry);
        }

        let modules_list = Widget::with_theme(ListBox::new(entries), "modules_list");

        let modules_list_ref = Rc::clone(&modules_list);
        let cb: Callback<Button> = Callback::new(Rc::new(move |_, _| {
            for child in modules_list_ref.borrow().children.iter() {
                if child.borrow().state.animation_state.contains(animation_state::Kind::Active) {
                    trace!("Found active module");
                    EXIT.with(|exit| *exit.borrow_mut() = true);
                }
            }
        }));
        let play = Widget::with_theme(Button::with_callback(cb), "play_button");
        play.borrow_mut().state.animation_state.add(animation_state::Kind::Disabled);

        vec![title, modules_title, play, modules_list]
    }
}
