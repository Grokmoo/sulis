use std::rc::Rc;
use std::cell::RefCell;

use grt::ui::{MarkupRenderer, TextArea, Widget, WidgetKind};
use grt::io::{GraphicsRenderer};
use grt::util::Point;

use state::{ChangeListener, EntityState};

const NAME: &'static str = "entity_mouseover";

pub struct EntityMouseover {
    entity: Rc<RefCell<EntityState>>,
    text_area: Rc<TextArea>,
}

impl EntityMouseover {
    pub fn new(entity: &Rc<RefCell<EntityState>>) -> Rc<EntityMouseover> {
        Rc::new(EntityMouseover {
            entity: Rc::clone(entity),
            text_area: TextArea::empty(),
        })
    }
}

impl WidgetKind for EntityMouseover {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_remove(&self) {
        //self.entity.borrow_mut().actor.listeners.remove(NAME);
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.entity.borrow_mut().actor.listeners.add(
            ChangeListener::invalidate_layout(NAME, widget));

        Vec::new()
    }

    fn layout(&self, widget: &mut Widget) {
        info!("Layout entity mouseover");
        widget.state.add_text_arg("name", &self.entity.borrow().actor.actor.name);
        widget.state.add_text_arg("cur_hp", &self.entity.borrow().actor.hp().to_string());
        widget.state.add_text_arg("max_hp", &self.entity.borrow().actor.stats.max_hp.to_string());

        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(
                    MarkupRenderer::new(font, widget.state.inner_size.width)));
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.text_area.draw_graphics_mode(renderer, pixel_size, widget, millis);
    }
}
