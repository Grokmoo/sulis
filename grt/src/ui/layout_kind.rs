use std::rc::Rc;
use std::cell::RefMut;
use std::cmp;

use ui::{Size, Widget};
use ui::theme::SizeRelative;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LayoutKind {
    Normal,
    BoxVertical,
    BoxHorizontal,
}

impl LayoutKind {
    pub fn layout(&self, widget: &Widget) {
        use self::LayoutKind::*;
        match *self {
            Normal => LayoutKind::layout_normal(widget),
            BoxVertical => LayoutKind::layout_box_vertical(widget),
            BoxHorizontal => LayoutKind::layout_box_horizontal(widget),
        }
    }

    fn layout_box_horizontal(widget: &Widget) {
        let height = widget.state.inner_size.height;
        let y = widget.state.inner_top();
        let mut current_x = widget.state.inner_left();

        for child in widget.children.iter() {
            let width = LayoutKind::get_preferred_width_recursive(&child.borrow_mut());
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(current_x, y);
            current_x += width;
        }
    }

    fn layout_box_vertical(widget: &Widget) {
        let width = widget.state.inner_size.width;
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            let height = LayoutKind::get_preferred_height_recursive(&child.borrow_mut());
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
        }
    }

    fn layout_normal(widget: &Widget) {
        for child in widget.children.iter() {
            LayoutKind::child_layout_normal(widget, &mut child.borrow_mut());
        }
    }

    fn child_layout_normal(widget: &Widget, child: &mut RefMut<Widget>) {
        let theme = match child.theme {
            None => return,
            Some(ref t) => Rc::clone(&t),
        };

        let mut size = Size::new(LayoutKind::get_preferred_width_recursive(child),
            LayoutKind::get_preferred_height_recursive(child));
        if theme.width_relative == SizeRelative::Max {
            size.add_width(widget.state.inner_size.width);
        }

        if theme.height_relative == SizeRelative::Max {
            size.add_height(widget.state.inner_size.height);
        }

        size.min_from(&widget.state.inner_size);
        child.state.set_size(size);

        use ui::theme::PositionRelative::*;
        let x = match theme.x_relative {
            Zero => widget.state.inner_left(),
            Center => (widget.state.inner_left() + widget.state.inner_right() -
                       size.width) / 2,
            Max => widget.state.inner_right() - size.width,
            Cursor => ::ui::Cursor::get_x(),
        };
        let y = match theme.y_relative {
            Zero => widget.state.inner_top(),
            Center => (widget.state.inner_top() + widget.state.inner_bottom() -
                       size.height) / 2,
            Max => widget.state.inner_bottom() - size.height,
            Cursor => ::ui::Cursor::get_y(),
        };

        child.state.set_position(
            x + theme.position.x, y + theme.position.y);
    }

    fn get_preferred_height_recursive(widget: &RefMut<Widget>) -> i32 {
        let theme = match widget.theme {
            None => return 0,
            Some(ref t) => t,
        };

        let mut height = 0;

        use ui::theme::SizeRelative::*;
        match theme.height_relative {
            ChildMax => {
                for child in widget.children.iter() {
                    height = cmp::max(height, LayoutKind::get_preferred_height_recursive(&child.borrow_mut()));
                }
                height += theme.border.vertical()
            },
            ChildSum => {
                for child in widget.children.iter() {
                    height += LayoutKind::get_preferred_height_recursive(&child.borrow_mut());
                }
                height += theme.border.vertical()
            },
            _ => {},
        };

        height + theme.preferred_size.height
    }

    fn get_preferred_width_recursive(widget: &RefMut<Widget>) -> i32 {
        let theme = match widget.theme {
            None => return 0,
            Some(ref t) => t,
        };

        let mut width = 0;

        use ui::theme::SizeRelative::*;
        match theme.width_relative {
            ChildMax => {
                for child in widget.children.iter() {
                    width = cmp::max(width, LayoutKind::get_preferred_width_recursive(&child.borrow_mut()));
                }
                width += theme.border.horizontal()
            },
            ChildSum => {
                for child in widget.children.iter() {
                    width += LayoutKind::get_preferred_width_recursive(&child.borrow_mut());
                }
                width += theme.border.horizontal()
            },
            _ => {},
        };

        width + theme.preferred_size.width
    }

}
