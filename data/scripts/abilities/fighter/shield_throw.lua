function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end

  stats = parent:inventory():equipped_stats("held_off")
  local dists = {
    light = 13,
	medium = 8,
	heavy = 5
  }
  
  targets = parent:targets():hostile():visible_within(dists[stats.armor_kind])
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  dur = 10 / (200 * game:anim_base_time())
  
  gen = parent:create_image_layer_anim(2 * dur)
  gen:add_image("HeldOff", "empty")
  gen:activate()
  
  x0 = parent:x()
  y0 = parent:y()
  x1 = target:x()
  y1 = target:y()

  anim = parent:create_anim("spinning_shield", dur)
  anim:set_position(anim:param(x0, -x0 / dur, x1 / (dur * dur)),
                    anim:param(y0, -y0 / dur, y1 / (dur * dur))
				   )
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  anim:add_callback(cb, dur)
  anim:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()
  
  stats = parent:stats()
  min_dmg = 10 + stats.dexterity_bonus / 2 + stats.level
  max_dmg = 20 + stats.dexterity_bonus / 2 + stats.level
  parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 5, "Piercing")
  
  dur = 10 / (200 * game:anim_base_time())
  
  x1 = parent:x()
  y1 = parent:y()
  x0 = target:x()
  y0 = target:y()

  anim = parent:create_anim("spinning_shield", dur)
  anim:set_position(anim:param(x0, x1 / dur, -x0 / (dur * dur)),
                    anim:param(y0, y1 / dur, -y0 / (dur * dur))
				   )
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:activate()
end
