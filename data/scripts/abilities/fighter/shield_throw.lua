function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end
  
  local targets = parent:targets():hostile():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local dur = 10 / (200 * game:anim_base_time())
  
  local gen = parent:create_image_layer_anim(2 * dur)
  gen:add_image("HeldOff", "empty")
  gen:activate()
  
  local x0 = parent:x()
  local y0 = parent:y()
  local x1 = target:x()
  local y1 = target:y()

  local anim = parent:create_anim("spinning_shield", dur)
  anim:set_position(anim:param(x0, -x0 / dur, x1 / (dur * dur)),
                    anim:param(y0, -y0 / dur, y1 / (dur * dur))
				   )
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  anim:add_callback(cb, dur)
  anim:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  
  local stats = parent:stats()
  local min_dmg = 10 + stats.dexterity_bonus / 2 + stats.level
  local max_dmg = 20 + stats.dexterity_bonus / 2 + stats.level
  local hit = parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 5, "Piercing")
  if hit:is_miss() then
    game:play_sfx("sfx/swish_2")
  elseif hit:is_graze() then
    game:play_sfx("sfx/thwack-03")
  elseif hit:is_hit() then
    game:play_sfx("sfx/hit_3")
  elseif hit:is_crit() then
    game:play_sfx("sfx/hit_2")
  end
  
  local dur = 10 / (200 * game:anim_base_time())
  
  local x1 = parent:x()
  local y1 = parent:y()
  local x0 = target:x()
  local y0 = target:y()

  local anim = parent:create_anim("spinning_shield", dur)
  anim:set_position(anim:param(x0, x1 / dur, -x0 / (dur * dur)),
                    anim:param(y0, y1 / dur, -y0 / (dur * dur))
				   )
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:activate()
end
