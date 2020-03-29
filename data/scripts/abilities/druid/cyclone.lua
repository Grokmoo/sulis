function on_activate(parent, ability)
  local size = "6by6round"
  local offset = 2
  if parent:ability_level(ability) > 1 then
    size = "8by8round"
	offset = 3
  end

  local targets = parent:targets():without_self()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:impass_blocks_affected_points(false)
  targeter:set_shape_line(size, parent:x() - offset, parent:y() - offset, ability:range())
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  
  local speed = 250 * game:anim_base_time()
  local duration = ability:range() / speed

  local delta_x = pos.x - parent:x()
  local delta_y = pos.y - parent:y()
  local angle = game:atan2(delta_x, delta_y)
  
  local norm = math.sqrt((delta_x * delta_x) + (delta_y * delta_y))
  
  delta_x = delta_x / norm * ability:range()
  delta_y = delta_y / norm * ability:range()
  
  local size = 8.0
  if parent:ability_level(ability) > 1 then
    size = 10.0
  end
  
  local anim = parent:create_anim("cyclone", duration)
  anim:set_position(
    anim:param(parent:x() - size / 2.0, delta_x / duration),
    anim:param(parent:y() - size / 2.0 + 1.0, delta_y / duration)
  )
  anim:set_particle_size_dist(anim:fixed_dist(size), anim:fixed_dist(size))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0))
  
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
  game:play_sfx("sfx/wind", 1.3)
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  local stats = parent:stats()
  local min_dmg = 18 + stats.caster_level / 2 + stats.wisdom_bonus / 4
  local max_dmg = 28 + stats.wisdom_bonus / 2 + stats.caster_level
  
  local hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 7, "Crushing")
  
  local base_dist = math.floor(8 + stats.caster_level / 3, stats.wisdom_bonus / 6 - target:width())
  if parent:ability_level(ability) > 1 then
    base_dist = base_dist + 3
  end
  
  local direction = -1
  
  for i = 1, 5 do
    local point = pick_random_point(target:x(), target:y())
    local dist = push_target(base_dist, target, hit, point, direction)
	if dist > 0 then break end
  end
end

function pick_random_point(x, y)
  return {x = x + math.random(-5, 5), y = y + math.random(-5, 5)}
end

--INCLUDE push_target