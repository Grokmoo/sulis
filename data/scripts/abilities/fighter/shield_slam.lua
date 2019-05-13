function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end

  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_reachable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_stun_effect")
 
  if parent:ability_level(ability) > 1 then
    local effect = parent:create_effect(ability:name(), 0)
    effect:add_num_bonus("melee_accuracy", 25 + parent:stats().level)
    effect:apply()
  end
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_stun_effect(parent, ability, targets, hit)
  local target = targets:first()
  
  -- compute the max target pushback distance
  local pushback_dist = 2 + parent:width() - target:width()
 
  if parent:ability_level(ability) > 1 then
    pushback_dist = pushback_dist + 3
  end
  
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    target:change_overflow_ap(-1000)
	pushback_dist = pushback_dist - 2
  elseif hit:is_hit() then
    target:change_overflow_ap(-2000)
  elseif hit:is_crit() then
    target:change_overflow_ap(-3000)
	pushback_dist = pushback_dist + 2
  end
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
  
  if pushback_dist < 1 then
    return
  end
  
  -- compute the normalized direction to push
  local target_x = target:x()
  local target_y = target:y()
  local dir_x = target_x - parent:x()
  local dir_y = target_y - parent:y()
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
