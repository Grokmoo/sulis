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

use std::collections::HashMap;
use std::str::FromStr;

use rlua::{Context, UserData, UserDataMethods};

use crate::animation::Anim;
use crate::script::{CallbackData, Result};
use crate::GameState;
use sulis_core::{resource::ResourceSet, util::ExtInt};
use sulis_module::ImageLayer;

/// An animation that adds one or more ImageLayers to the parent creature
/// for rendering.  These override any racial or inventory image layers, with
/// "empty" being used to hide an image layer on the parent.  All layers are
/// removed when the animation is complete.
///
/// # `activate()`
/// Activates and applies this animation to the parent.
///
/// # `add_image(layer: String, image: String)`
/// Adds the specified image for the specified layer.  An image with this ID must exist.
/// Valid ImageLayers are HeldMain, HeldOff, Ears, Hair, Beard, Head, Hands,
/// Foreground, Torso, Legs, Feet, Background, Cloak, Shadow
///
/// # `set_completion_callback(callback: CallbackData)`
/// Sets the specified `callback` to be called when this animation completes.
///
/// # `add_callback(callback: CallbackData, time: Float)`
/// Sets the specified `callback` to be called after the specified `time` has elapsed,
/// in seconds.
#[derive(Clone)]
pub struct ScriptImageLayerAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_millis: ExtInt,
    images: HashMap<ImageLayer, String>,
}

impl ScriptImageLayerAnimation {
    pub fn new(parent: usize, duration_millis: ExtInt) -> ScriptImageLayerAnimation {
        ScriptImageLayerAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_millis,
            images: HashMap::new(),
        }
    }
}

impl UserData for ScriptImageLayerAnimation {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", &activate);
        methods.add_method_mut("add_image", |_, gen, (layer, image): (String, String)| {
            let layer = match ImageLayer::from_str(&layer) {
                Ok(layer) => layer,
                Err(error) => {
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ImageLayer",
                        message: Some(format!("{}", error)),
                    });
                }
            };

            if ResourceSet::image(&image).is_none() {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "Image",
                    message: Some(format!("No image with ID '{}'", image)),
                });
            };

            gen.images.insert(layer, image);
            Ok(())
        });
        methods.add_method_mut("set_completion_callback", |_, gen, cb: CallbackData| {
            gen.completion_callback = Some(cb);
            Ok(())
        });
        methods.add_method_mut("add_callback", |_, gen, (cb, time): (CallbackData, f32)| {
            gen.callbacks.push((time, cb));
            Ok(())
        });
    }
}

fn activate(_lua: Context, data: &ScriptImageLayerAnimation, _args: ()) -> Result<()> {
    let anim = create_anim(data)?;

    GameState::add_animation(anim);

    Ok(())
}

pub fn create_anim(data: &ScriptImageLayerAnimation) -> Result<Anim> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(data.parent);

    let mut images = HashMap::new();
    for (layer, ref image_id) in data.images.iter() {
        match ResourceSet::image(image_id) {
            None => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "Image",
                    message: Some(format!("No image with ID '{}'", image_id)),
                });
            }
            Some(image) => images.insert(*layer, image),
        };
    }

    let mut anim = Anim::new_entity_image_layer(&parent, data.duration_millis, images);

    if let Some(ref cb) = data.completion_callback {
        anim.add_completion_callback(Box::new(cb.clone()));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_update_callback(Box::new(cb.clone()), (time * 1000.0) as u32);
    }

    Ok(anim)
}
