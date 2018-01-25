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
use std::cell::RefCell;

use ui::Widget;

pub struct Callback {
    cb: Rc<Fn(&Rc<RefCell<Widget>>)>
}

impl Callback {
    pub fn new(f: Rc<Fn(&Rc<RefCell<Widget>>)>) -> Callback {
        Callback {
            cb: f,
        }
    }

    pub fn with(f: Box<Fn()>) -> Callback {
        Callback {
            cb: Rc::new(move |_w| { f() }),
        }
    }

    pub fn remove_self() -> Callback {
        Callback {
            cb: Rc::new(|widget| { widget.borrow_mut().mark_for_removal() }),
        }
    }

    pub fn remove_parent() -> Callback {
        Callback {
            cb: Rc::new(|widget| {
                let parent = Widget::get_parent(&widget);
                parent.borrow_mut().mark_for_removal();
            }),
        }
    }

    pub fn call(&self, widget: &Rc<RefCell<Widget>>) {
        (self.cb)(widget);
    }

    pub fn clone(&self) -> Callback {
        Callback {
            cb: Rc::clone(&self.cb),
        }
    }
}
