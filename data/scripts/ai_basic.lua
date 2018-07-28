function ai_action(parent, state)
  if attempt_attack(parent) then
    game:log("returning state from attack")
    return
  end

  if attempt_move(parent) then
    game:log("returning state from move")
	_G.state = parent:state_wait(10)
    return
  end
  
  game:log("returning end")
  _G.state = parent:state_end()
end

function attempt_move(parent)
  if not parent:can_move() then
    return false
  end

  targets = parent:targets():hostile():visible()
  
  closest_dist = 1000
  closest_target = nil
  
  targets = targets:to_table()
  for i = 1, #targets do
    target = targets[i]
    dist = parent:dist_to_entity(target)
	if not parent:can_reach(target) then
	  if dist < closest_dist then
	    closest_dist = dist
	    closest_target = target
	  end
	end
  end

  if closest_target ~= nil then
    parent:move_towards_entity(closest_target)
	return true
  else
    return false
  end
end

function attempt_attack(parent)
  targets = parent:targets():hostile():attackable()
  
  targets = targets:to_table()
  for i = 1, #targets do
    target = targets[i]
	parent:anim_weapon_attack(target, nil, true)
	if parent:has_ap_to_attack() then
	  _G.state = parent:state_wait(10)
	else
	  _G.state = parent:state_end()
	end
	return true
  end
  
  return false
end