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

use std::rc::Rc;
use std::cell::RefMut;
use std::cmp;

use ui::{Size, Widget};
use ui::theme::Theme;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LayoutKind {
    Normal,
    BoxVertical,
    BoxHorizontal,
    Grid,
}

impl LayoutKind {
    pub fn layout(&self, widget: &Widget) {
        let theme = match widget.theme {
            None => return,
            Some(ref t) => Rc::clone(&t),
        };

        use self::LayoutKind::*;
        match *self {
            Normal => LayoutKind::layout_normal(widget),
            BoxVertical => LayoutKind::layout_box_vertical(widget, theme),
            BoxHorizontal => LayoutKind::layout_box_horizontal(widget, theme),
            Grid => LayoutKind::layout_grid(widget, theme),
        }
    }

    fn layout_grid(widget: &Widget, theme: Rc<Theme>) {
        let mut current_x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        let mut max_height = 0;
        current_y += theme.layout_spacing.top;

        for child in widget.children.iter() {
            current_x += theme.layout_spacing.left;

            let width = LayoutKind::get_preferred_width_recursive(&child.borrow_mut(),
                &widget.state.inner_size);
            let height = LayoutKind::get_preferred_height_recursive(&child.borrow_mut(),
                &widget.state.inner_size);
            max_height = cmp::max(max_height, height);
            child.borrow_mut().state.set_size(Size::new(width, height));

            if current_x + width + theme.layout_spacing.right > widget.state.inner_right() {
                current_x = widget.state.inner_left() + theme.layout_spacing.left;
                current_y += max_height;
                current_y += theme.layout_spacing.top + theme.layout_spacing.bottom;
                max_height = 0;
            }

            child.borrow_mut().state.set_position(current_x, current_y);
            current_x += width + theme.layout_spacing.right;
        }
    }

    fn layout_box_horizontal(widget: &Widget, theme: Rc<Theme>) {
        let height = widget.state.inner_size.height;
        let y = widget.state.inner_top();
        let mut current_x = widget.state.inner_left();

        for child in widget.children.iter() {
            current_x += theme.layout_spacing.left;
            let width = LayoutKind::get_preferred_width_recursive(&child.borrow_mut(),
                &widget.state.inner_size);
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(current_x, y);
            current_x += width;
            current_x += theme.layout_spacing.right;
        }
    }

    fn layout_box_vertical(widget: &Widget, theme: Rc<Theme>) {
        let width = widget.state.inner_size.width;
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            current_y += theme.layout_spacing.top;
            let height = LayoutKind::get_preferred_height_recursive(&child.borrow_mut(),
                &widget.state.inner_size);
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
            current_y += theme.layout_spacing.bottom;
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

        let size = Size::new(LayoutKind::get_preferred_width_recursive(child, &widget.state.inner_size),
            LayoutKind::get_preferred_height_recursive(child, &widget.state.inner_size));

        child.state.set_size(size);

        use ui::theme::PositionRelative::*;
        let x = match theme.x_relative {
            Zero => widget.state.inner_left(),
            Center => (widget.state.inner_left() + widget.state.inner_right() -
                       size.width) / 2,
            Max => widget.state.inner_right() - size.width,
            Cursor => ::ui::Cursor::get_x(),
            Custom => child.state.position.x,
        };
        let y = match theme.y_relative {
            Zero => widget.state.inner_top(),
            Center => (widget.state.inner_top() + widget.state.inner_bottom() -
                       size.height) / 2,
            Max => widget.state.inner_bottom() - size.height,
            Cursor => ::ui::Cursor::get_y(),
            Custom => child.state.position.y,
        };

        child.state.set_position(x + theme.position.x, y + theme.position.y);
    }

    fn get_preferred_height_recursive(widget: &RefMut<Widget>, parent_inner_size: &Size) -> i32 {
        let theme = match widget.theme {
            None => return 0,
            Some(ref t) => t,
        };

        let mut height = theme.preferred_size.height;

        use ui::theme::SizeRelative::*;
        match theme.height_relative {
            Max => height += parent_inner_size.height,
            ChildMax => {
                for child in widget.children.iter() {
                    height = cmp::max(height,
                        LayoutKind::get_preferred_height_recursive(&child.borrow_mut(), parent_inner_size));
                }
                height += theme.border.vertical()
            },
            ChildSum => {
                for child in widget.children.iter() {
                    height +=
                        LayoutKind::get_preferred_height_recursive(&child.borrow_mut(), parent_inner_size);
                }
                height += theme.border.vertical()
            },
            _ => {},
        };

        height
    }

    fn get_preferred_width_recursive(widget: &RefMut<Widget>, parent_inner_size: &Size) -> i32 {
        let theme = match widget.theme {
            None => return 0,
            Some(ref t) => t,
        };

        let mut width = theme.preferred_size.width;

        use ui::theme::SizeRelative::*;
        match theme.width_relative {
            Max => width += parent_inner_size.width,
            ChildMax => {
                for child in widget.children.iter() {
                    width = cmp::max(width,
                        LayoutKind::get_preferred_width_recursive(&child.borrow_mut(), parent_inner_size));
                }
                width += theme.border.horizontal()
            },
            ChildSum => {
                for child in widget.children.iter() {
                    width +=
                        LayoutKind::get_preferred_width_recursive(&child.borrow_mut(), parent_inner_size);
                }
                width += theme.border.horizontal()
            },
            _ => {},
        };

        width
    }

}
