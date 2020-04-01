function on_activate(parent, ability)
  local targets = parent:targets():without_self():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:impass_blocks_affected_points(true)
  targeter:set_shape_line("1by1", parent:x(), parent:y(), ability:range())
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  
  local speed = 500 * game:anim_base_time()
  local duration = ability:range() / speed
  
  local anim = parent:create_particle_generator("fire_particle", duration + 0.2)

  local delta_x = pos.x - parent:x()
  local delta_y = pos.y - parent:y()
  local angle = game:atan2(delta_x, delta_y)
  
  local norm = math.sqrt((delta_x * delta_x) + (delta_y * delta_y))
  
  delta_x = delta_x / norm * ability:range()
  delta_y = delta_y / norm * ability:range()
  
  anim:set_position(anim:param(parent:x(), delta_x / duration), anim:param(parent:y(), delta_y / duration))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.3, 0.3), anim:fixed_dist(-delta_x / duration)),
    anim:dist_param(anim:uniform_dist(-0.3, 0.3), anim:fixed_dist(-delta_y / duration)))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_particle_duration_dist(anim:fixed_dist(0.6))
  anim:set_gen_rate(anim:param(500.0))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(1.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do 
    local dist = parent:dist_to_entity(targets[i])
    local duration = dist / speed
    
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    anim:add_callback(cb, duration)
  end
  
  anim:activate()
  ability:activate(parent)
  game:play_sfx("sfx/explode4")
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  local stats = parent:stats()
  local min_dmg = 8 + stats.caster_level / 2 + stats.intellect_bonus / 4
  local max_dmg = 16 + stats.intellect_bonus / 2 + stats.caster_level
  parent:special_attack(target, "Will", "Spell", min_dmg, max_dmg, 5, "Piercing")
  parent:special_attack(target, "Will", "Spell", min_dmg, max_dmg, 5, "Shock")
  
  game:play_sfx("sfx/explode5")
end
