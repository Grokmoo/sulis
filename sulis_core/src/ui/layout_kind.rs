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

use std::cmp;
use std::rc::Rc;

use crate::config::Config;
use crate::ui::theme::Theme;
use crate::ui::{Cursor, Size, Widget};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    Normal,
    BoxVertical,
    BoxHorizontal,
    GridRows,
    GridColumns,
}

impl Default for LayoutKind {
    fn default() -> Self {
        LayoutKind::Normal
    }
}

impl LayoutKind {
    pub fn layout(self, widget: &Widget) {
        let theme = Rc::clone(&widget.theme);
        use self::LayoutKind::*;
        match self {
            Normal => LayoutKind::layout_normal(widget, theme),
            BoxVertical => LayoutKind::layout_box_vertical(widget, theme),
            BoxHorizontal => LayoutKind::layout_box_horizontal(widget, theme),
            GridRows => LayoutKind::layout_grid(widget, theme),
            GridColumns => LayoutKind::layout_grid_column(widget, theme),
        }
    }

    fn layout_grid_column(widget: &Widget, theme: Rc<Theme>) {
        let mut current_x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        let mut max_width = 0;
        current_x += theme.layout_spacing.left;

        for child in widget.children.iter() {
            current_y += theme.layout_spacing.top;

            let width =
                LayoutKind::width_recursive(&child.borrow_mut(), widget.state.inner_width());
            let height =
                LayoutKind::height_recursive(&child.borrow_mut(), widget.state.inner_height());
            max_width = cmp::max(max_width, width);
            child.borrow_mut().state.set_size(Size::new(width, height));

            if current_y + height + theme.layout_spacing.bottom > widget.state.inner_bottom() + 1 {
                current_y = widget.state.inner_top() + theme.layout_spacing.top;
                current_x += max_width;
                current_x += theme.layout_spacing.left + theme.layout_spacing.right;
                max_width = 0;
            }

            child.borrow_mut().state.set_position(current_x, current_y);
            current_y += height + theme.layout_spacing.bottom;
        }
    }

    fn layout_grid(widget: &Widget, theme: Rc<Theme>) {
        let mut current_x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        let mut max_height = 0;
        current_y += theme.layout_spacing.top;

        for child in widget.children.iter() {
            current_x += theme.layout_spacing.left;

            let width =
                LayoutKind::width_recursive(&child.borrow_mut(), widget.state.inner_width());
            let height =
                LayoutKind::height_recursive(&child.borrow_mut(), widget.state.inner_height());
            max_height = cmp::max(max_height, height);
            child.borrow_mut().state.set_size(Size::new(width, height));

            if current_x + width + theme.layout_spacing.right > widget.state.inner_right() + 1 {
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
        let height = widget.state.inner_height();
        let y = widget.state.inner_top();
        let mut current_x = widget.state.inner_left();

        for child in widget.children.iter() {
            current_x += theme.layout_spacing.left;
            let width =
                LayoutKind::width_recursive(&child.borrow_mut(), widget.state.inner_width());
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(current_x, y);
            current_x += width;
            current_x += theme.layout_spacing.right;
        }
    }

    fn layout_box_vertical(widget: &Widget, theme: Rc<Theme>) {
        let width = widget.state.inner_width();
        let x = widget.state.inner_left();
        let mut current_y = widget.state.inner_top();

        for child in widget.children.iter() {
            current_y += theme.layout_spacing.top;
            let height =
                LayoutKind::height_recursive(&child.borrow_mut(), widget.state.inner_height());
            child.borrow_mut().state.set_size(Size::new(width, height));
            child.borrow_mut().state.set_position(x, current_y);
            current_y += height;
            current_y += theme.layout_spacing.bottom;
        }
    }

    fn layout_normal(widget: &Widget, _theme: Rc<Theme>) {
        for child in widget.children.iter() {
            LayoutKind::child_layout_normal(widget, &mut child.borrow_mut());
        }
    }

    fn child_layout_normal(widget: &Widget, child: &mut Widget) {
        let theme = Rc::clone(&child.theme);
        let size = Size::new(
            LayoutKind::width_recursive(child, widget.state.inner_width()),
            LayoutKind::height_recursive(child, widget.state.inner_height()),
        );

        child.state.set_size(size);

        use crate::ui::theme::PositionRelative::*;
        let x = match theme.relative.x {
            Zero => widget.state.inner_left(),
            Center => (widget.state.inner_left() + widget.state.inner_right() - size.width) / 2,
            Max => widget.state.inner_right() - size.width,
            Custom => child.state.position().x,
            Mouse => cmp::min(Cursor::get_x(), Config::ui_width() - size.width),
        };
        let y = match theme.relative.y {
            Zero => widget.state.inner_top(),
            Center => (widget.state.inner_top() + widget.state.inner_bottom() - size.height) / 2,
            Max => widget.state.inner_bottom() - size.height,
            Custom => child.state.position().y,
            Mouse => cmp::min(Cursor::get_y(), Config::ui_height() - size.height),
        };

        child
            .state
            .set_position(x + theme.position.x, y + theme.position.y);
    }

    fn height_recursive(widget: &Widget, parent_inner_height: i32) -> i32 {
        let theme = &widget.theme;
        let mut height = theme.size.height;

        use crate::ui::theme::SizeRelative::*;
        match theme.relative.height {
            Max => height += parent_inner_height,
            ChildMax => {
                for child in widget.children.iter() {
                    height = cmp::max(
                        height,
                        LayoutKind::height_recursive(&child.borrow(), parent_inner_height),
                    );
                }
                height += theme.border.vertical()
            }
            ChildSum => {
                for child in widget.children.iter() {
                    height += LayoutKind::height_recursive(&child.borrow(), parent_inner_height);
                }
                height += theme.border.vertical();

                if let LayoutKind::BoxVertical = theme.layout {
                    height += theme.layout_spacing.vertical() * (widget.children.len() as i32 - 1);
                }
            }
            Custom => height += widget.state.size().height,
            Zero => (),
        };

        height
    }

    fn width_recursive(widget: &Widget, parent_inner_width: i32) -> i32 {
        let theme = &widget.theme;
        let mut width = theme.size.width;

        use crate::ui::theme::SizeRelative::*;
        match theme.relative.width {
            Max => width += parent_inner_width,
            ChildMax => {
                for child in widget.children.iter() {
                    width = cmp::max(
                        width,
                        LayoutKind::width_recursive(&child.borrow(), parent_inner_width),
                    );
                }
                width += theme.border.horizontal()
            }
            ChildSum => {
                for child in widget.children.iter() {
                    width += LayoutKind::width_recursive(&child.borrow(), parent_inner_width);
                }
                width += theme.border.horizontal();

                if let LayoutKind::BoxHorizontal = theme.layout {
                    width += theme.layout_spacing.horizontal() * (widget.children.len() as i32 - 1);
                }
            }
            Custom => width += widget.state.size().width,
            Zero => (),
        };

        width
    }
}
