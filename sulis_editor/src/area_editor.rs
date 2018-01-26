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

use sulis_core::ui::WidgetKind;

const NAME: &str = "area_editor";

pub struct AreaEditor {
    width: i32,
    height: i32,
}

impl AreaEditor {
    pub fn new(width: i32, height: i32) -> Rc<AreaEditor> {
        Rc::new(AreaEditor {
            width,
            height,
        })
    }
}

impl WidgetKind for AreaEditor {
    fn get_name(&self) -> &str {
        NAME
    }
}
