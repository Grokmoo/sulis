function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end
  
  if parent:has_active_mode() then
    game:say_line("Only one mode may be active at a time.", parent)
    return
  end

  local stats = parent:stats()
  local amount = 5 + stats.level / 2

  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  local cb = ability:create_callback(parent)
  cb:set_after_defense_fn("after_defense")
  effect:add_callback(cb)

  local gen = parent:create_anim("shield_deflect")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.7), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.4), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function after_defense(parent, ability, targets, hit)
  if hit:total_damage() < 1 then return end

  local target = targets:first()

  if target:inventory():weapon_style() ~= "Ranged" then return end
  
  local max_dmg = hit:total_damage()
  local min_dmg = max_dmg / 2

  local stats = target:stats()
  local projectile = stats.ranged_projectile
  
  local dist = parent:dist_to_entity(target)
  local speed = 500 * game:anim_base_time()
  local duration = dist / speed
  local anim = parent:create_anim(projectile, duration)
  
  local delta_x = target:x() - parent:x()
  local delta_y = target:y() - parent:y()
  local angle = game:atan2(delta_x, delta_y)
  
  anim:set_position(anim:param(parent:x(), delta_x / duration), anim:param(parent:y(), delta_y / duration))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_rotation(anim:param(angle))
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  anim:add_callback(cb, duration)
  anim:activate()
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  local stats = target:stats()
  parent:special_attack(target, "Reflex", "Ranged", stats.damage_min_0, stats.damage_max_0, stats.armor_penetration_0, "Piercing")
end