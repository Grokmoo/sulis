-- This file is included by abilities that push a target in a given direction

function push_target(base_dist, target, hit, point, direction)
  local pushback_dist = base_dist
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
    return 0
  end
  
  local max_dist = target:dist_to_point(point) + 3
  if pushback_dist > max_dist then
    pushback_dist = max_dist
  end
  
  -- compute the normalized direction to push
  local target_x = target:x()
  local target_y = target:y()
  local dir_x = (target_x - point.x) * direction
  local dir_y = (target_y - point.y) * direction
  local mag = math.sqrt(dir_x * dir_x + dir_y * dir_y)
  
  -- don't divide by 0
  if math.abs(mag) < 0.1 then return 0 end
  
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
    return 0
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
  
  return total_dist
end