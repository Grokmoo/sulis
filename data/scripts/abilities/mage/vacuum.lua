radius = 7.0

function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  targeter:set_selection_radius(15.0)
  targeter:set_shape_circle(radius)
  targeter:invis_blocks_affected_points(true)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local position = targets:selected_point()
  
  local anim = parent:create_anim("wind_collapse", 0.7)
  anim:set_position(anim:param(position.x - 6.0), anim:param(position.y - 6.0))
  anim:set_particle_size_dist(anim:fixed_dist(12.0), anim:fixed_dist(12.0))
  anim:activate()
  
  local gen = parent:wait_anim(0.7)
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:add_selected_point(position)
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, 0.5 * (radius - targets[i]:dist_to_point(position)) / radius)
  end
  gen:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  
  if not target:is_valid() then return end

  local stats = parent:stats()
  local min_dmg = 15 + stats.caster_level / 3 + stats.intellect_bonus / 6
  local max_dmg = 25 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
  local hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 20, "Crushing")
  
  push_target(parent, target, hit, targets:selected_point())
end

function push_target(parent, target, hit, point)
  local stats = parent:stats()

  local pushback_dist = math.floor(8 + stats.caster_level / 3 + stats.intellect_bonus / 6 - target:width())
  if hit:is_miss() then
    pushback_dist = pushback_dist - 4
  elseif hit:is_graze() then
    pushback_dist = pushback_dist - 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    pushback_dist = pushback_dist + 2
  end
  
  if pushback_dist < 1 then
    return
  end
  
  local max_dist = target:dist_to_point(point) + 3
  if pushback_dist > max_dist then
    pushback_dist = max_dist
  end
  
  -- compute the normalized direction to push
  local target_x = target:x()
  local target_y = target:y()
  local dir_x = point.x - target_x
  local dir_y = point.y - target_y
  local mag = math.sqrt(dir_x * dir_x + dir_y * dir_y)
  local x_norm = dir_x / mag
  local y_norm = dir_y / mag
  
  local dest_x = target_x
  local dest_y = target_y
  
  local total_dist = 0
  -- go along the direction, checking until we hit an impassable spot
  for dist = 1, pushback_dist do
    local test_x = math.floor(target_x + x_norm * dist + 0.5)
	local test_y = math.floor(target_y + y_norm * dist + 0.5)
	
	if not game:is_passable(target, test_x, test_y) then
	  break
	end
	
	dest_x = test_x
	dest_y = test_y
	total_dist = dist
  end
  
  -- return if the result is to not move the target
  if dest_x == target_x and dest_y == target_y then
    return
  end
  local dest = { x = dest_x, y = dest_y }
  
  -- move the target now (since we know the dest is valid now) and hide it with a subpos animation
  target:teleport_to(dest)
  
  local subpos_x = dest_x - target_x
  local subpos_y = dest_y - target_y
  target:set_subpos(-subpos_x, -subpos_y)
  
  -- create the movement animation for the computed destination
  local speed = 300 * game:anim_base_time()
  local duration = total_dist / speed
  local anim = target:create_subpos_anim(duration)
  anim:set_position(anim:param(-subpos_x, subpos_x / duration), anim:param(-subpos_y, subpos_y / duration))
  anim:activate()
end