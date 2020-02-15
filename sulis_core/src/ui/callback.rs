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

use crate::ui::{Widget, WidgetKind, RcRfc};

type CallbackFn = dyn Fn(&RcRfc<Widget>, &mut dyn WidgetKind);

pub struct Callback {
    cb: Rc<CallbackFn>,
}

impl Clone for Callback {
    fn clone(&self) -> Callback {
        Callback {
            cb: Rc::clone(&self.cb),
        }
    }
}

impl Callback {
    pub fn empty() -> Callback {
        Callback {
            cb: Rc::new(|_, _| {}),
        }
    }

    pub fn new(f: Rc<CallbackFn>) -> Callback {
        Callback { cb: f }
    }

    pub fn with_widget(f: Rc<dyn Fn(&RcRfc<Widget>)>) -> Callback {
        Callback {
            cb: Rc::new(move |widget, _kind| f(widget)),
        }
    }

    pub fn with(f: Box<dyn Fn()>) -> Callback {
        Callback {
            cb: Rc::new(move |_w, _k| f()),
        }
    }

    pub fn invalidate_self_layout() -> Callback {
        Callback {
            cb: Rc::new(|widget, _kind| widget.borrow_mut().invalidate_layout()),
        }
    }

    pub fn remove_self() -> Callback {
        Callback {
            cb: Rc::new(|widget, _kind| widget.borrow_mut().mark_for_removal()),
        }
    }

    pub fn call(&self, widget: &RcRfc<Widget>, kind: &mut dyn WidgetKind) {
        (self.cb)(widget, kind);
    }
}
