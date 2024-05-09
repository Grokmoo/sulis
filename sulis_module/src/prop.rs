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

use std::io::Error;
use std::rc::Rc;

use serde::Deserialize;

use sulis_core::image::Image;
use sulis_core::io::DrawList;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::AnimationState;
use sulis_core::util::{unable_to_create_error, Offset, Point, Rect};

use crate::area::tile::verify_point;
use crate::{LootList, Module, ObjectSize, OnTrigger};

#[derive(Debug)]
pub enum Interactive {
    Not,
    Container {
        loot: Option<Rc<LootList>>,
    },
    Door {
        initially_open: bool,
        closed_impass: Vec<Point>,
        closed_invis: Vec<Point>,
        on_activate: Vec<OnTrigger>,
        fire_more_than_once: bool,
    },
    Hover,
}

#[derive(Debug)]
pub struct Prop {
    pub id: String,
    pub name: String,
    pub icon: Rc<dyn Image>,
    pub image: Rc<dyn Image>,
    pub random_millis_offset: u32,
    pub size: Rc<ObjectSize>,
    pub impass: Vec<Point>,
    pub invis: Vec<Point>,
    pub interactive: Interactive,
    pub aerial: bool,
    pub status_text: Option<String>,
}

impl Prop {
    pub fn new(builder: PropBuilder, module: &Module) -> Result<Prop, Error> {
        let icon = match ResourceSet::image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return unable_to_create_error("prop", &builder.id);
            }
            Some(icon) => icon,
        };

        let image = match ResourceSet::image(&builder.image) {
            None => {
                warn!("No image found for image '{}'", builder.image);
                return unable_to_create_error("prop", &builder.id);
            }
            Some(image) => image,
        };

        let size = match module.sizes.get(&builder.size) {
            None => {
                warn!("No size found with id '{}'", builder.size);
                return unable_to_create_error("prop", &builder.id);
            }
            Some(size) => Rc::clone(size),
        };

        if builder.passable.is_some() && builder.impass.is_some() {
            warn!("Cannot specify both overall passable and impass array");
            return unable_to_create_error("prop", &builder.id);
        }

        if builder.visible.is_some() && builder.invis.is_some() {
            warn!("Cannot specify both overall visible and invis array");
            return unable_to_create_error("prop", &builder.id);
        }

        let mut impass = Vec::new();
        if let Some(pass) = builder.passable {
            if !pass {
                for y in 0..size.height {
                    for x in 0..size.width {
                        impass.push(Point::new(x, y));
                    }
                }
            }
        } else if let Some(builder_impass) = builder.impass {
            for p in builder_impass {
                let (x, y) = verify_point("impass", size.width as usize, size.height as usize, p)?;
                impass.push(Point::new(x, y));
            }
        }

        let mut invis = Vec::new();
        if let Some(vis) = builder.visible {
            if !vis {
                for y in 0..size.height {
                    for x in 0..size.width {
                        invis.push(Point::new(x, y));
                    }
                }
            }
        } else if let Some(builder_invis) = builder.invis {
            for p in builder_invis {
                let (x, y) = verify_point("invis", size.width as usize, size.height as usize, p)?;
                invis.push(Point::new(x, y));
            }
        }

        let interactive = match builder.interactive {
            InteractiveBuilder::Not => Interactive::Not,
            InteractiveBuilder::Hover => Interactive::Hover,
            InteractiveBuilder::Container { loot } => {
                let loot = match loot {
                    None => None,
                    Some(loot) => match module.loot_lists.get(&loot) {
                        None => {
                            warn!("Unable to find loot list '{}'", loot);
                            return unable_to_create_error("prop", &builder.id);
                        }
                        Some(loot) => Some(Rc::clone(loot)),
                    },
                };
                Interactive::Container { loot }
            }
            InteractiveBuilder::Door {
                initially_open,
                closed_impass,
                closed_invis,
                on_activate,
                fire_more_than_once,
            } => Interactive::Door {
                initially_open,
                closed_impass,
                closed_invis,
                on_activate,
                fire_more_than_once,
            },
        };

        Ok(Prop {
            id: builder.id,
            name: builder.name,
            icon,
            image,
            random_millis_offset: builder.random_millis_offset,
            size,
            impass,
            invis,
            interactive,
            aerial: builder.aerial,
            status_text: builder.status_text,
        })
    }

    pub fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        state: &AnimationState,
        offset: Offset,
        millis: u32,
    ) {
        let rect = Rect {
            x: offset.x,
            y: offset.y,
            w: self.size.width as f32,
            h: self.size.height as f32,
        };

        self.image
            .append_to_draw_list(draw_list, state, rect, millis);
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum InteractiveBuilder {
    Not,
    Container {
        loot: Option<String>,
    },
    Door {
        initially_open: bool,
        closed_impass: Vec<Point>,
        closed_invis: Vec<Point>,

        #[serde(default)]
        on_activate: Vec<OnTrigger>,

        #[serde(default)]
        fire_more_than_once: bool,
    },
    Hover,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PropBuilder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub image: String,
    #[serde(default)]
    pub random_millis_offset: u32,
    pub size: String,
    pub passable: Option<bool>,
    pub impass: Option<Vec<Vec<usize>>>,
    pub invis: Option<Vec<Vec<usize>>>,
    pub visible: Option<bool>,
    #[serde(default)]
    pub aerial: bool,
    pub interactive: InteractiveBuilder,
    pub status_text: Option<String>,
}
