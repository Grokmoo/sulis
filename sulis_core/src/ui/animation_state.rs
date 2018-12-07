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

use std::io::{Error, ErrorKind};

use lazy_static::lazy_static;

use self::Kind::*;

#[derive(PartialOrd, Ord, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Normal,
    Hover,
    Pressed,
    Active,
    Disabled,

    Custom1,
    Custom2,
    Custom3,
    Custom4,
    Custom5,

    MouseMove,
    MouseLoot,
    MouseAttack,
    MouseTravel,
    MouseSelect,
    MouseInvalid,
    MouseDialog,
    MouseSearch,
    MouseInteract,
    MouseUnlock,
}

impl Kind {
    pub fn get_text(&self) -> &str {
        match *self {
            Normal => "normal",
            Hover => "hover",
            Pressed => "pressed",
            Active => "active",
            Disabled => "disabled",
            Custom1 => "custom1",
            Custom2 => "custom2",
            Custom3 => "custom3",
            Custom4 => "custom4",
            Custom5 => "custom5",
            MouseMove => "mouse_move",
            MouseLoot => "mouse_loot",
            MouseAttack => "mouse_attack",
            MouseTravel => "mouse_travel",
            MouseSelect => "mouse_select",
            MouseInvalid => "mouse_invalid",
            MouseDialog => "mouse_dialog",
            MouseSearch => "mouse_search",
            MouseUnlock => "mouse_unlock",
            MouseInteract => "mouse_interact",
        }
    }

    pub fn find(text: &str) -> Option<Kind> {
        Some(match text {
            "normal" => Normal,
            "hover" => Hover,
            "pressed" => Pressed,
            "active" => Active,
            "disabled" => Disabled,
            "custom1" => Custom1,
            "custom2" => Custom2,
            "custom3" => Custom3,
            "custom4" => Custom4,
            "custom5" => Custom5,
            "mouse_move" => MouseMove,
            "mouse_loot" => MouseLoot,
            "mouse_attack" => MouseAttack,
            "mouse_travel" => MouseTravel,
            "mouse_select" => MouseSelect,
            "mouse_invalid" => MouseInvalid,
            "mouse_dialog" => MouseDialog,
            "mouse_search" => MouseSearch,
            "mouse_unlock" => MouseUnlock,
            "mouse_interact" => MouseInteract,
            _ => return None,
        })
    }
}

lazy_static! {
    pub static ref NORMAL: AnimationState = AnimationState::default();
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AnimationState {
    kinds: Vec<Kind>,
}

impl AnimationState {
    /// Adds the specified kind if it is not present, removes it
    /// if it is.  See `add` and `remove`
    pub fn toggle(&mut self, kind: Kind) {
        if !self.kinds.contains(&kind) {
            self.add(kind);
        } else {
            self.remove(kind);
        }
    }

    /// Adds the specified kind to this animation state, if it is not
    /// already present.  Removes `Kind::Normal` if it is present, as
    /// it may only ever be in an otherwise empty AnimationState
    pub fn add(&mut self, kind: Kind) {
        self.kinds.retain(|k| *k != Kind::Normal);

        if !self.kinds.contains(&kind) {
            self.kinds.push(kind);
            self.kinds.sort_unstable();
        }
    }

    /// Removes the specified kind from this animation state, if it
    /// is present.  If the AnimationState would be empty after this,
    /// adds `Kind::Normal`
    pub fn remove(&mut self, kind: Kind) {
        self.kinds.retain(|k| *k != kind);
        if self.kinds.len() == 0 {
            self.kinds.push(Kind::Normal);
        }
    }

    /// Returns true if the state contains this kind, false otherwise
    pub fn contains(&self, kind: Kind) -> bool {
        self.kinds.contains(&kind)
    }
}

impl AnimationState {
    /// Creates an instance of the default animation state, containing
    /// `Kind::Normal`
    pub fn default() -> AnimationState {
        AnimationState {
            kinds: vec![Kind::Normal]
        }
    }

    /// Creates a new AnimationState containing only the given `kind`
    pub fn with(kind: Kind) -> AnimationState {
        AnimationState {
            kinds: vec![kind]
        }
    }

    /// Creates an animation state by parsing the given string.  The string
    /// consists of any number of `Kind` string identifiers, separated by the
    /// '+' character and optional whitespace.  An empty state (with no valid
    /// Kinds) is not allowed.  `Kind::Normal` may not be present with any
    /// other Kinds.  Duplicate Kinds are not allowed.
    ///
    /// # Examples
    /// ```
    /// use ui::AnimationState;
    /// let state = AnimationState::parse("hover + active");
    /// let state2 = AnimationState::parse("pressed");
    /// ```
    pub fn parse(data: &str) -> Result<AnimationState, Error> {
        let mut kinds: Vec<Kind> = Vec::with_capacity(1);
        for split in data.split('+').map(|s| s.trim()) {
            match Kind::find(split) {
                Some(kind) => AnimationState::validate_push(kind, &mut kinds)?,
                None => return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Unable to parse animation state from '{}'", data))),
            };
        }

        if kinds.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Empty AnimationState string"));
        }

        return Ok(AnimationState {
            kinds,
        })
    }

    fn validate_push(kind: Kind, kinds: &mut Vec<Kind>) -> Result<(), Error> {
        if kinds.contains(&kind) {
            return Err(Error::new(ErrorKind::InvalidData,
                                  format!("Duplicate key '{:?}'", kind)));
        }

        if kind == Kind::Normal && kinds.len() > 0 {
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Normal state cannot be present with others."));
        } else if kinds.contains(&Kind::Normal) {
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Normal state cannot be present with others."));
        }

        kinds.push(kind);
        kinds.sort_unstable();
        Ok(())
    }

    /// Finds the `AnimationState` within the set of `choices` that closest
    /// matches the `desired_state`.  Each animation state is ranked according to
    /// what fraction of its kinds match the desired state kind, and what fraction
    /// of the desired state's kinds match the animation state's kind.  The state
    /// with the highest rank is chosen and the associated `T` is returned.
    pub fn find_match_in_vec<'a, T>(desired_state: &'a AnimationState,
                             choices: &'a Vec<(AnimationState, T)>) -> &'a T {

        // TODO each animation state is sorted, so we can do this with a single
        // lock step loop instead of double loop
        let num_desired_kinds = desired_state.kinds.len() as f32;
        let mut best_index = 0;
        let mut best_rank = 0.0;
        for (index, &(ref state, _)) in choices.iter().enumerate() {
            let mut cur_rank = 0.0;
            for kind in desired_state.kinds.iter() {
                if state.contains(*kind) { cur_rank += 1.0; }
            }

            cur_rank = cur_rank / num_desired_kinds + cur_rank / (state.kinds.len() as f32);
            if cur_rank > best_rank {
                best_index = index;
                best_rank = cur_rank;

                if best_rank > 1.99 { break; }
            }
        }

        &choices[best_index].1
    }
}
