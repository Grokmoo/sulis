function ai_action(parent, state)
  if not parent:has_ap_to_attack() then
    _G.state = parent:state_end()
	return
  end

  if attempt_attack(parent) then
    _G.state = parent:state_wait(10)
    return
  end

  if attempt_move(parent) then
    _G.state = parent:state_wait(10)
    return
  end

  _G.state = parent:state_end()
end

function attempt_move(parent)
  if not parent:can_move() then
    return false
  end

  targets = parent:targets():hostile()
  
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
  targets = parent:targets():hostile():visible():attackable()
  
  targets = targets:to_table()
  for i = 1, #targets do
	attack_or_ability(parent, targets[i])
	return true
  end
  
  return false
end

function attack_or_ability(parent, target)
  abilities = parent:abilities():can_activate():to_table()
  
  for i = 1, #abilities do
    parent:use_ability(abilities[i])
	handle_targeter(parent)
    return
  end

  parent:anim_weapon_attack(target, nil, true)
end

function handle_targeter(parent)
  targets = parent:targets():visible()
  
  targets = targets:to_table()
  for i = 1, #targets do
    target = targets[i]
	x = target:x()
	y = target:y()
    if game:check_targeter_position(x, y) then
	  game:activate_targeter()
	  return
	end
  end
  
  game:cancel_targeter()
end