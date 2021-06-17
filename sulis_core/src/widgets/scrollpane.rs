//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::any::Any;
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;

use crate::io::{GraphicsRenderer, InputAction, event::ClickKind};
use crate::ui::{Callback, Widget, WidgetKind, WidgetState};
use crate::util::{Point, Size};
use crate::widget_kind;

use crate::widgets::{Button, TextArea};

#[derive(Copy, Clone)]
pub enum ScrollDirection {
    Vertical,
    Horizontal,
}

/// Simple ScrollPane that clips its child content to its inner area
/// nested scroll panes are not supported
/// only does vertical scrolling
pub struct ScrollPane {
    content: Rc<RefCell<Widget>>,
    content_size: i32,
    scrollbar: Rc<RefCell<Scrollbar>>,
    scrollbar_widget: Rc<RefCell<Widget>>,
    scroll_direction: ScrollDirection,
}

impl ScrollPane {
    pub fn new(direction: ScrollDirection) -> Rc<RefCell<ScrollPane>> {
        let content = Widget::empty("content");
        let scrollbar = Scrollbar::new(direction, &content);

        let scrollbar_widget = Widget::with_defaults(scrollbar.clone());
        Rc::new(RefCell::new(ScrollPane {
            content,
            content_size: 0,
            scrollbar,
            scrollbar_widget,
            scroll_direction: direction,
        }))
    }

    pub fn add_to_content(&self, child: Rc<RefCell<Widget>>) {
        // for text areas inside scroll panes don't limit drawing
        // inside the screen area on layout as scroll panes
        // need to allow widgets to be off screen
        disable_text_area_limit_recursive(&child);

        Widget::add_child_to(&self.content, child);
    }

    fn layout_vertical(&mut self, widget: &mut Widget) {
        let scroll = self.scrollbar.borrow().cur_pos();

        widget.do_self_layout();
        let (x, y) = widget.state.position().as_tuple();
        widget.state.set_position(x, y - scroll);

        widget.do_children_layout();

        widget.state.set_position(x, y);

        let (sx, sy) = self.scrollbar_widget.borrow().state.position().as_tuple();
        self.scrollbar_widget
            .borrow_mut()
            .state
            .set_position(sx, sy + scroll);

        if self.content_size == 0 {
            self.content_size = self.content.borrow().state.height();
            self.scrollbar
                .borrow_mut()
                .set_content_size(self.content_size);
        }

        let w = self.content.borrow().state.width();
        self.content
            .borrow_mut()
            .state
            .set_size(Size::new(w, self.content_size + scroll));
    }

    fn layout_horizontal(&mut self, widget: &mut Widget) {
        let scroll = self.scrollbar.borrow().cur_pos();

        widget.do_self_layout();

        let (x, y) = widget.state.position().as_tuple();
        widget.state.set_position(x - scroll, y);
        widget.do_children_layout();
        widget.state.set_position(x, y);

        let (sx, sy) = self.scrollbar_widget.borrow().state.position().as_tuple();
        self.scrollbar_widget
            .borrow_mut()
            .state
            .set_position(sx + scroll, sy);

        if self.content_size == 0 {
            self.content_size = self.content.borrow().state.width();
            self.scrollbar
                .borrow_mut()
                .set_content_size(self.content_size);
        }

        let h = self.content.borrow().state.height();
        self.content
            .borrow_mut()
            .state
            .set_size(Size::new(self.content_size + scroll, h));
    }
}

fn disable_text_area_limit_recursive(parent: &Rc<RefCell<Widget>>) {
    let kind = Rc::clone(&parent.borrow().kind);
    match kind.borrow_mut().as_any_mut().downcast_mut::<TextArea>() {
        None => (),
        Some(text_area) => {
            text_area.limit_to_screen_edge = false;
        }
    }

    for child in parent.borrow().children.iter() {
        disable_text_area_limit_recursive(child);
    }
}

impl WidgetKind for ScrollPane {
    widget_kind!["scrollpane"];

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        _pixel_size: Point,
        widget: &Widget,
        _millis: u32,
    ) {
        let x = widget.state.inner_left();
        let y = widget.state.inner_top();
        let width = widget.state.inner_width();
        let height = widget.state.inner_height();
        renderer.set_scissor(Point::new(x, y), Size::new(width, height));
    }

    fn on_key_press(&mut self, _widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        let delta = match key {
            InputAction::ZoomIn => -1,
            InputAction::ZoomOut => 1,
            _ => return false,
        };

        self.scrollbar
            .borrow_mut()
            .update_children_position(&self.content, delta);

        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        match self.scroll_direction {
            ScrollDirection::Vertical => self.layout_vertical(widget),
            ScrollDirection::Horizontal => self.layout_horizontal(widget),
        }
    }

    fn end_draw(&mut self, renderer: &mut dyn GraphicsRenderer) {
        renderer.clear_scissor();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        vec![Rc::clone(&self.content), Rc::clone(&self.scrollbar_widget)]
    }
}

struct Scrollbar {
    direction: ScrollDirection,
    widget: Rc<RefCell<Widget>>,
    content_size: i32,
    delta: u32,

    min_pos: i32,
    cur_pos: i32,
    max_pos: i32,

    up: Rc<RefCell<Widget>>,
    down: Rc<RefCell<Widget>>,
    thumb: Rc<RefCell<Widget>>,

    thumb_accum: f32,
}

impl Scrollbar {
    fn new(
        direction: ScrollDirection,
        widget_to_scroll: &Rc<RefCell<Widget>>,
    ) -> Rc<RefCell<Scrollbar>> {
        let up = Widget::with_theme(Button::empty(), "up");
        let down = Widget::with_theme(Button::empty(), "down");
        let thumb = Widget::with_theme(ScrollThumb::new(), "thumb");

        Rc::new(RefCell::new(Scrollbar {
            direction,
            widget: Rc::clone(widget_to_scroll),
            delta: 1,
            cur_pos: 0,
            min_pos: 0,
            max_pos: 0,
            up,
            down,
            thumb,
            content_size: 0,
            thumb_accum: 0.0,
        }))
    }

    fn set_content_size(&mut self, size: i32) {
        self.content_size = size;
    }

    fn cur_pos(&self) -> i32 {
        self.cur_pos
    }

    fn update_children_position(&mut self, parent: &Rc<RefCell<Widget>>, dir: i32) {
        match self.direction {
            ScrollDirection::Vertical => self.compute_min_max_y(&parent.borrow()),
            ScrollDirection::Horizontal => self.compute_min_max_x(&parent.borrow()),
        }

        self.adjust_cur_pos(dir * self.delta as i32);

        let pane = Widget::direct_parent(&self.widget);
        pane.borrow_mut().invalidate_layout();
    }

    fn adjust_cur_pos(&mut self, amount: i32) {
        self.cur_pos += amount;

        if self.cur_pos < self.min_pos {
            self.cur_pos = self.min_pos
        } else if self.cur_pos > self.max_pos {
            self.cur_pos = self.max_pos
        }
    }

    fn compute_min_max_x(&mut self, widget: &Widget) {
        let mut max_x = 0;

        for child in &widget.children {
            let child = &child.borrow();
            max_x = cmp::max(max_x, child.state.left() + child.state.width());
        }

        let state = &widget.state;
        self.max_pos = max_x - self.content_size + state.border().horizontal() - state.left();
        if self.max_pos < self.min_pos {
            self.max_pos = self.min_pos
        }
    }

    fn compute_min_max_y(&mut self, widget: &Widget) {
        let mut max_y = 0;

        for child in &widget.children {
            let child = &child.borrow();

            max_y = cmp::max(max_y, child.state.top() + child.state.height());
        }

        let state = &widget.state;
        self.max_pos = max_y - self.content_size + state.border().vertical() - state.top();

        if self.max_pos < self.min_pos {
            self.max_pos = self.min_pos
        }
    }

    fn drag_thumb(&mut self, scrollable_size: i32, delta: f32) {
        let ratio = (self.max_pos - self.min_pos) as f32 * delta / (scrollable_size as f32);

        // accumlate any floating point error over multiple frames
        let accum = ratio + self.thumb_accum;
        self.thumb_accum = accum.fract();
        let adjust = (accum - accum.fract()) as i32;

        self.adjust_cur_pos(adjust);
    }

    fn set_thumb_state_vertical(&mut self, state: &WidgetState, thumb_frac: f32) {
        let inner_height = state.inner_height()
            - self.up.borrow().state.height()
            - self.down.borrow().state.height();
        let thumb_height = (thumb_frac * inner_height as f32).round() as i32;
        let thumb_width = self.thumb.borrow().state.width();
        self.thumb
            .borrow_mut()
            .state
            .set_size(Size::new(thumb_width, thumb_height));

        let thumb_x = self.thumb.borrow().state.left();
        let thumb_base_y = self.up.borrow().state.top() + self.up.borrow().state.height();

        let thumb_pos_frac = (self.cur_pos as f32 - self.min_pos as f32)
            / (self.max_pos as f32 - self.min_pos as f32);
        let thumb_max_y = self.down.borrow().state.top() - thumb_height - thumb_base_y;

        let thumb_y = thumb_base_y + (thumb_pos_frac * thumb_max_y as f32).round() as i32;

        self.thumb.borrow_mut().state.set_position(thumb_x, thumb_y);
    }

    fn set_thumb_state_horizontal(&mut self, state: &WidgetState, thumb_frac: f32) {
        let thumb = &mut self.thumb.borrow_mut();

        let inner_width =
            state.inner_width() - self.up.borrow().state.width() - self.down.borrow().state.width();
        let thumb_width = (thumb_frac * inner_width as f32).round() as i32;
        let thumb_height = thumb.state.height();
        thumb.state.set_size(Size::new(thumb_width, thumb_height));

        let thumb_y = thumb.state.top();
        let thumb_base_x = self.up.borrow().state.left() + self.up.borrow().state.width();

        let thumb_pos_frac = (self.cur_pos as f32 - self.min_pos as f32)
            / (self.max_pos as f32 - self.min_pos as f32);
        let thumb_max_x = self.down.borrow().state.left() - thumb_width - thumb_base_x;

        let thumb_x = thumb_base_x + (thumb_pos_frac * thumb_max_x as f32).round() as i32;

        thumb.state.set_position(thumb_x, thumb_y);
    }
}

impl WidgetKind for Scrollbar {
    widget_kind!("scrollbar");

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_base_layout();

        if self.content_size == 0 {
            self.content_size = self.widget.borrow().state.height();
        }

        let widget_ref = Rc::clone(&self.widget);
        match self.direction {
            ScrollDirection::Vertical => self.compute_min_max_y(&widget_ref.borrow()),
            ScrollDirection::Horizontal => self.compute_min_max_x(&widget_ref.borrow()),
        }

        self.delta = widget.theme.get_custom_or_default("scroll_delta", 1);

        self.up
            .borrow_mut()
            .state
            .set_enabled(self.cur_pos != self.min_pos);
        self.down
            .borrow_mut()
            .state
            .set_enabled(self.cur_pos != self.max_pos);

        let widget_size = self.content_size as f32;
        let thumb_frac = widget_size / ((self.max_pos as f32 - self.min_pos as f32) + widget_size);

        //self.thumb.borrow_mut().state.set_enabled(false);
        widget.state.set_visible(thumb_frac < 1.0);

        match self.direction {
            ScrollDirection::Vertical => self.set_thumb_state_vertical(&widget.state, thumb_frac),
            ScrollDirection::Horizontal => {
                self.set_thumb_state_horizontal(&widget.state, thumb_frac)
            }
        }
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let widget_ref = Rc::clone(&self.widget);
        self.up
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let (_, kind) = Widget::parent_mut::<Scrollbar>(widget);
                kind.update_children_position(&widget_ref, -1);
            })));

        let widget_ref = Rc::clone(&self.widget);
        self.down
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _| {
                let (_, kind) = Widget::parent_mut::<Scrollbar>(widget);
                kind.update_children_position(&widget_ref, 1);
            })));

        vec![
            Rc::clone(&self.up),
            Rc::clone(&self.down),
            Rc::clone(&self.thumb),
        ]
    }
}

struct ScrollThumb {

}

impl ScrollThumb {
    fn new() -> Rc<RefCell<ScrollThumb>> {
        Rc::new(RefCell::new(ScrollThumb { }))
    }
}

impl WidgetKind for ScrollThumb {
    widget_kind!("scrollthumb");

    fn on_mouse_drag(
        &mut self,
        widget: &Rc<RefCell<Widget>>,
        _kind: ClickKind,
        delta_x: f32,
        delta_y: f32,
    ) -> bool {
        let (parent, bar) = Widget::parent_mut::<Scrollbar>(widget);

        let parent_parent = Widget::direct_parent(&parent);
        parent_parent.borrow_mut().invalidate_layout();

        Widget::get_root(&parent_parent).borrow_mut().set_mouse_drag_child(widget);

        match bar.direction {
            ScrollDirection::Horizontal => {
                let mut scrollable_width = parent.borrow().state.inner_width();
                // subtract width of the left/right buttons as well as the thumb
                for child in parent.borrow().children.iter() {
                    scrollable_width -= child.borrow().state.width();
                }

                bar.drag_thumb(scrollable_width, delta_x);
            }, ScrollDirection::Vertical => {
                let mut scrollable_height = parent.borrow().state.inner_height();
                // subtract height of the up/down buttons as well as the thumb
                for child in parent.borrow().children.iter() {
                    scrollable_height -= child.borrow().state.height();
                }

                bar.drag_thumb(scrollable_height, delta_y);
            }
        }

        true
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);

        Widget::get_root(widget).borrow_mut().clear_mouse_drag_child(widget);

        true
    }
}