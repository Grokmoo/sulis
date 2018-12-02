max_dist = 8

function on_activate(parent, ability)
  targets = parent:targets():hostile()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(max_dist * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), max_dist, math.pi / 3) 
  targeter:add_all_effectable(targets)
  targeter:impass_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  if targets:is_empty() then return end
  target = targets:first()
  
  speed = 500 * game:anim_base_time()
  dist = parent:dist_to_entity(target)
  duration = 0.2 + dist / speed
  
  anim = parent:create_subpos_anim(duration)

  delta_x = target:x() - parent:x()
  delta_y = target:y() - parent:y()
  
  anim:set_position(anim:param(0.0, delta_x / duration), anim:param(0.0, delta_y / duration))
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("attack_target1")
  anim:set_completion_callback(cb)
  
  anim:activate()
  ability:activate(parent)
end

function attack_target1(parent, ability, targets)
  target1 = targets:first()

  if target1:is_valid() then
    parent:weapon_attack(target1)
  end

  hide = parent:get_ability("hide")
  cb = hide:create_callback(parent)
  cb:set_on_anim_complete_fn("deactivate")
  hide_anim = parent:wait_anim(0.1)
  hide_anim:set_completion_callback(cb)
  hide_anim:activate()
  
  targets_table = targets:to_table()
  if #targets_table < 2 then
    anim_return(parent, ability, target1)
    return
  end
  
  initial_delta_x = target1:x() - parent:x()
  initial_delta_y = target1:y() - parent:y()
  
  target2 = targets_table[2]
  
  speed = 500 * game:anim_base_time()
  dist = target1:dist_to_entity(target2)
  duration = 0.2 + dist / speed
  parent:set_subpos(initial_delta_x, initial_delta_y)
  
  anim = parent:create_subpos_anim(duration)

  delta_x = target2:x() - target1:x()
  delta_y = target2:y() - target1:y()
  
  anim:set_position(anim:param(initial_delta_x, delta_x / duration), anim:param(initial_delta_y, delta_y / duration))
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("attack_target2")
  anim:set_completion_callback(cb)
  
  anim:activate()
end

function attack_target2(parent, ability, targets)
  targets_table = targets:to_table()
  target2 = targets_table[2]

  if target2:is_valid() then
    parent:weapon_attack(target2)
  end

  if #targets_table < 3 then
    anim_return(parent, ability, target2)
    return
  end
  
  initial_delta_x = target2:x() - parent:x()
  initial_delta_y = target2:y() - parent:y()
  
  parent:set_subpos(initial_delta_x, initial_delta_y)
  target3 = targets_table[3]
  
  speed = 500 * game:anim_base_time()
  dist = target2:dist_to_entity(target3)
  duration = 0.2 + dist / speed
  
  anim = parent:create_subpos_anim(duration)

  delta_x = target3:x() - target2:x()
  delta_y = target3:y() - target2:y()
  
  anim:set_position(anim:param(initial_delta_x, delta_x / duration), anim:param(initial_delta_y, delta_y / duration))
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("attack_target3")
  anim:set_completion_callback(cb)
  
  anim:activate()
end

function attack_target3(parent, ability, targets)
  targets = targets:to_table()
  target3 = targets[3]

  if target3:is_valid() then
    parent:weapon_attack(target3)
  end
  
  anim_return(parent, ability, target3)
end

function anim_return(parent, ability, target)
  speed = 500 * game:anim_base_time()
  dist = target3:dist_to_entity(target)
  duration = 0.2 + dist / speed
  
  anim = parent:create_subpos_anim(duration)
  
  delta_x = target:x() - parent:x()
  delta_y = target:y() - parent:y()
  
  parent:set_subpos(delta_x, delta_y)
  anim:set_position(anim:param(delta_x, -delta_x / duration), anim:param(delta_y, -delta_y / duration))
  anim:activate()
end