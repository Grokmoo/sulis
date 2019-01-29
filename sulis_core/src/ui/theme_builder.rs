//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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
use std::collections::HashMap;
use std::str::FromStr;
use std::io::{Error, ErrorKind};

use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;

use crate::ui::{Border, Color, LayoutKind, theme::*};
use crate::util::{Size, Point};

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct RelativeBuilder {
    x: Option<PositionRelative>,
    y: Option<PositionRelative>,
    width: Option<SizeRelative>,
    height: Option<SizeRelative>,
}

impl RelativeBuilder {
    fn build(self) -> Relative {
        Relative {
            x: self.x.unwrap_or_default(),
            y: self.y.unwrap_or_default(),
            width: self.width.unwrap_or_default(),
            height: self.height.unwrap_or_default(),
        }
    }

    fn merge(to: &mut Option<RelativeBuilder>, from: &Option<RelativeBuilder>) {
        let from = match from {
            None => return,
            Some(from) => from,
        };

        match to {
            None => {
                to.replace(from.clone());
            },
            Some(to) => {
                to.x = to.x.or(from.x);
                to.y = to.y.or(from.y);
                to.width = to.width.or(from.width);
                to.height = to.height.or(from.height);
            }
        }
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TextParamsBuilder {
    horizontal_alignment: Option<HorizontalAlignment>,
    vertical_alignment: Option<VerticalAlignment>,

    #[serde(default, deserialize_with="de_color")]
    color: Option<Color>,

    scale: Option<f32>,
    font: Option<String>,
}

impl TextParamsBuilder {
    fn build(self) -> TextParams {
        TextParams {
            horizontal_alignment: self.horizontal_alignment.unwrap_or_default(),
            vertical_alignment: self.vertical_alignment.unwrap_or_default(),
            color: self.color.unwrap_or_default(),
            scale: self.scale.unwrap_or(1.0),
            font: self.font.unwrap_or("normal".to_string()),
        }
    }

    fn merge(to: &mut Option<TextParamsBuilder>, from: &Option<TextParamsBuilder>) {
        let from = match from {
            None => return,
            Some(from) => from,
        };

        match to {
            None => {
                to.replace(from.clone());
            }, Some(to) => {
                to.horizontal_alignment = to.horizontal_alignment.or(from.horizontal_alignment);
                to.vertical_alignment = to.vertical_alignment.or(from.vertical_alignment);
                if to.color.is_none() { to.color = from.color.clone(); }
                to.scale = to.scale.or(from.scale);
                if to.font.is_none() { to.font = from.font.clone(); }
            }
        }
    }
}

fn de_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where D:Deserializer<'de> {
    let input: Option<String> = Option::deserialize(deserializer)?;

    Ok(match input {
        None => None,
        Some(input) => {
            use serde::de::Error;
            let color = Color::from_str(&input).map_err(|err| Error::custom(err.to_string()))?;
            Some(color)
        }
    })
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ThemeBuilder {
    from: Option<String>,
    kind: Option<Kind>,
    layout: Option<LayoutKind>,
    layout_spacing: Option<Border>,
    border: Option<Border>,
    size: Option<Size>,
    position: Option<Point>,
    relative: Option<RelativeBuilder>,

    text: Option<String>,
    text_params: Option<TextParamsBuilder>,
    background: Option<String>,
    foreground: Option<String>,

    #[serde(default)]
    custom: HashMap<String, String>,

    #[serde(default)]
    children: HashMap<String, ThemeBuilder>,

    #[serde(skip)]
    children_ids: Vec<String>,
    #[serde(skip)]
    parent_id: Option<String>,
}

impl ThemeBuilder {
    fn build(self, id: String) -> Theme {
        Theme {
            id,
            layout: self.layout.unwrap_or_default(),
            layout_spacing: self.layout_spacing.unwrap_or_default(),
            border: self.border.unwrap_or_default(),
            size: self.size.unwrap_or_default(),
            position: self.position.unwrap_or_default(),
            relative: match self.relative {
                None => Relative::default(),
                Some(rb) => rb.build(),
            },
            text: self.text,
            text_params: match self.text_params {
                None => TextParams::default(),
                Some(pb) => pb.build(),
            },
            background: self.background,
            foreground: self.foreground,
            custom: self.custom,
            kind: self.kind.unwrap_or_default(),
            children: self.children_ids,
            parent: self.parent_id,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeBuilderSet {
    pub(crate) id: String,
    pub(crate) themes: HashMap<String, ThemeBuilder>,
}

const MAX_FROM_DEPTH: u32 = 10;

impl ThemeBuilderSet {
    pub fn create_theme_set(mut self) -> Result<ThemeSet, Error> {
        self.flatten_children()?;
        self.expand_from()?;

        let mut out = HashMap::new();
        for (id, builder) in self.themes {
            let id_clone = id.clone();
            out.insert(id_clone, Rc::new(builder.build(id)));
        }

        // the default theme in the default location to look
        out.insert(DEFAULT_THEME_ID.to_string(), Rc::new(Theme::default()));

        Ok(ThemeSet::new(out))
    }

    fn flatten_children(&mut self) -> Result<(), Error> {
        let ids: Vec<_> = self.themes.keys().map(|k| k.to_string()).collect();

        for id in ids {
            self.flatten_children_recursive(id)?;
        }

        Ok(())
    }

    fn flatten_children_recursive(&mut self, id: String) -> Result<(), Error> {
        let children: Vec<(String, ThemeBuilder)> =
            self.themes.get_mut(&id).unwrap().children.iter()
                .map(|(k, v)| (k.to_string(), v.clone())).collect();

        for (child_id, mut child) in children {
            let new_id = format!("{}.{}", id, child_id);

            if self.themes.contains_key(&new_id) {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Computed ID '{}' is already present", new_id)));
            }

            child.parent_id = Some(id.clone());
            self.themes.insert(new_id.clone(), child);

            self.themes.get_mut(&id).unwrap().children_ids.push(new_id.clone());

            self.flatten_children_recursive(new_id)?;
        }

        Ok(())
    }

    fn expand_from(&mut self) -> Result<(), Error> {
        // this set is different from the previous step as children have been added
        let ids: Vec<_> = self.themes.keys().map(|k| k.to_string()).collect();

        for id in ids {
            self.expand_from_recursive(&id, 0)?;
        }
        Ok(())
    }

    fn expand_from_recursive(&mut self, id: &str, depth: u32) -> Result<(), Error> {
        if depth > MAX_FROM_DEPTH {
            return Err(Error::new(ErrorKind::InvalidInput,
                                  format!("From reference depth exceeds max of {}. This is most\
                                          likely caused by a circular reference.", MAX_FROM_DEPTH)));
        }

        if !self.themes.contains_key(id) {
            return Err(Error::new(ErrorKind::InvalidInput,
                                  format!("From Reference '{}' is invalid", id)));
        }

        let from = self.themes.get_mut(id).unwrap().from.take();
        if let Some(from) = from {
            self.expand_from_recursive(&from, depth + 1)?;

            // TODO more efficient method that doesn't clone the whole set
            // to satisfy the borrow checker.  should be possible with unsafe -
            // we need a mutable reference to a themes element and a shared
            // reference at the same time, but as long as they aren't equal
            // it should be ok
            let from_theme = self.themes[&from].clone();
            self.copy_from_theme_recursive(id, &from_theme, depth)?;
        }
        Ok(())
    }

    fn copy_from_theme_recursive(&mut self, id: &str, from: &ThemeBuilder,
                                 depth: u32) -> Result<(), Error> {
        if !self.themes.contains_key(id) {
            self.themes.insert(id.to_string(), from.clone());
        } else {
            ThemeBuilderSet::copy_from_theme(&mut self.themes.get_mut(id).unwrap(), &from);
        }

        for (child_id, child) in from.children.iter() {
            // TODO expand the child from if it exists
            let sub_id = format!("{}.{}", id, child_id);
            self.copy_from_theme_recursive(&sub_id, child, depth)?;
        }

        Ok(())
    }

    fn copy_from_theme(to: &mut ThemeBuilder, from: &ThemeBuilder) {
        to.layout = to.layout.or(from.layout);
        to.layout_spacing = to.layout_spacing.or(from.layout_spacing);
        to.border = to.border.or(from.border);
        to.size = to.size.or(from.size);
        to.position = to.position.or(from.position);
        to.kind = to.kind.or(from.kind);

        if to.text.is_none() { to.text = from.text.clone(); }
        if to.background.is_none() { to.background = from.background.clone(); }
        if to.foreground.is_none() { to.foreground = from.foreground.clone(); }

        RelativeBuilder::merge(&mut to.relative, &from.relative);
        TextParamsBuilder::merge(&mut to.text_params, &from.text_params);

        for (key, value) in from.custom.iter() {
            to.custom.entry(key.to_string()).or_insert(value.to_string());
        }
    }

}

