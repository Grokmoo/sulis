function on_activate(parent, ability)
  local dist = 4.0 + parent:ability_level(ability) * 2.0

  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(dist)
  targeter:set_free_select(dist)
  targeter:set_free_select_must_be_passable(parent:size_str())
  targeter:set_shape_object_size(parent:size_str())
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  
  local speed = 600 * game:anim_base_time()
  local dist = parent:dist_to_point(pos)
  local duration = dist / speed
  
  local hide = parent:get_ability("hide")
  local cb = hide:create_callback(parent)
  cb:set_on_anim_complete_fn("activate_no_ap")
  local hide_anim = parent:wait_anim(duration + 0.5)
  hide_anim:set_completion_callback(cb)
  hide_anim:activate()
  
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("move_parent")
  
  local anim = parent:create_subpos_anim(duration)

  local delta_x = pos.x - parent:x()
  local delta_y = pos.y - parent:y()
  
  anim:set_position(anim:param(0.0, delta_x / duration), anim:param(0.0, delta_y / duration))
  anim:set_completion_callback(cb)
  anim:activate()
  ability:activate(parent)
end

function move_parent(parent, ability, targets)
  local dest = targets:selected_point()
  parent:teleport_to(dest)
end
