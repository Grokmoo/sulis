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

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

use crate::ui::{theme::*, Border, Color, LayoutKind};
use crate::util::{Point, Size};

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

    fn merge(to: &mut Option<RelativeBuilder>, from: Option<RelativeBuilder>) {
        let from = match from {
            None => return,
            Some(from) => from,
        };

        match to {
            None => {
                to.replace(from.clone());
            }
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

    #[serde(default, deserialize_with = "de_color")]
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
            font: self.font.unwrap_or_else(|| "normal".to_string()),
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
            }
            Some(to) => {
                to.horizontal_alignment = to.horizontal_alignment.or(from.horizontal_alignment);
                to.vertical_alignment = to.vertical_alignment.or(from.vertical_alignment);
                if to.color.is_none() {
                    to.color = from.color;
                }
                to.scale = to.scale.or(from.scale);
                if to.font.is_none() {
                    to.font = from.font.clone();
                }
            }
        }
    }
}

fn de_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
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

    fn expand_from(
        &mut self,
        themes: &HashMap<String, ThemeBuilder>,
        depth: u32,
    ) -> Result<(), Error> {
        if depth > MAX_FROM_DEPTH {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "From reference depth exceeds max of {}. This is most\
                     likely caused by a circular reference.",
                    MAX_FROM_DEPTH
                ),
            ));
        }

        self.expand_self(themes, depth)?;

        for (_, child) in self.children.iter_mut() {
            child.expand_from(themes, depth + 1)?;
        }

        Ok(())
    }

    fn expand_self(
        &mut self,
        themes: &HashMap<String, ThemeBuilder>,
        depth: u32,
    ) -> Result<(), Error> {
        if depth > MAX_FROM_DEPTH {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "From reference depth exceeds max of {}. This is most\
                     likely caused by a circular reference.",
                    MAX_FROM_DEPTH
                ),
            ));
        }

        // recursively expand from definitions for this to
        loop {
            let from_id = match self.from.take() {
                None => break,
                Some(from) => from,
            };

            let from = ThemeBuilderSet::find_theme(themes, &from_id)?;
            let mut from = from.clone();
            from.expand_self(themes, depth + 1)?;

            ThemeBuilderSet::copy_from_theme(self, &from);
        }
        Ok(())
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
        self.expand_from()?;

        self.flatten_children()?;

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
            self.themes.get_mut(&id).unwrap().children.drain().collect();

        for (child_id, mut child) in children {
            let new_id = format!("{}.{}", id, child_id);

            if self.themes.contains_key(&new_id) {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Computed ID '{}' is already present", new_id),
                ));
            }

            child.parent_id = Some(id.clone());
            self.themes.insert(new_id.clone(), child);

            self.themes
                .get_mut(&id)
                .unwrap()
                .children_ids
                .push(new_id.clone());

            self.flatten_children_recursive(new_id)?;
        }

        Ok(())
    }

    fn expand_from(&mut self) -> Result<(), Error> {
        // take a copy of the whole tree to take the from refs from
        let clone = self.themes.clone();

        for (_, theme) in self.themes.iter_mut() {
            theme.expand_from(&clone, 0)?;
        }

        Ok(())
    }

    fn find_theme<'a>(
        themes: &'a HashMap<String, ThemeBuilder>,
        id: &str,
    ) -> Result<&'a ThemeBuilder, Error> {
        let mut parent = themes;
        let mut result = None;
        for part in id.split('.') {
            parent = match parent.get(part) {
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("From ref '{}' is invalid", id),
                    ));
                }
                Some(child) => {
                    result = Some(child);
                    &child.children
                }
            };
        }

        match result {
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("From ref '{}' is invalid", id),
            )),
            Some(theme) => Ok(theme),
        }
    }

    fn copy_from_theme(to: &mut ThemeBuilder, from: &ThemeBuilder) {
        // copy over the from from so that we follow a recursive from tree
        // to.from has already been set to None prior to calling this method
        to.from = from.from.clone();
        to.layout = to.layout.or(from.layout);
        to.layout_spacing = to.layout_spacing.or(from.layout_spacing);
        to.border = to.border.or(from.border);
        to.size = to.size.or(from.size);
        to.position = to.position.or(from.position);
        to.kind = to.kind.or(from.kind);

        if to.text.is_none() {
            to.text = from.text.clone();
        }
        if to.background.is_none() {
            to.background = from.background.clone();
        }
        if to.foreground.is_none() {
            to.foreground = from.foreground.clone();
        }

        RelativeBuilder::merge(&mut to.relative, from.relative);
        TextParamsBuilder::merge(&mut to.text_params, &from.text_params);

        for (key, value) in from.custom.iter() {
            to.custom
                .entry(key.to_string())
                .or_insert_with(|| value.to_string());
        }

        for (from_id, from_child) in from.children.iter() {
            match to.children.get_mut(from_id) {
                None => {
                    to.children.insert(from_id.clone(), from_child.clone());
                }
                Some(to_child) => {
                    ThemeBuilderSet::copy_from_theme(to_child, from_child);
                }
            }
        }
    }
}
