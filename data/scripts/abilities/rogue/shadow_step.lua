function on_activate(parent, ability)
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(10.0)
  targeter:set_free_select_must_be_passable(parent:size_str())
  targeter:set_shape_object_size(parent:size_str())
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  pos = targets:selected_point()
  
  speed = 600 * game:anim_base_time()
  dist = parent:dist_to_point(pos)
  duration = dist / speed
  
  hide = parent:get_ability("hide")
  cb = hide:create_callback(parent)
  cb:set_on_anim_complete_fn("activate_no_ap")
  hide_anim = parent:wait_anim(duration + 0.5)
  hide_anim:set_completion_callback(cb)
  hide_anim:activate()
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("move_parent")
  
  anim = parent:create_subpos_anim(duration)

  delta_x = pos.x - parent:x()
  delta_y = pos.y - parent:y()
  
  anim:set_position(anim:param(0.0, delta_x / duration), anim:param(0.0, delta_y / duration))
  anim:set_completion_callback(cb)
  anim:activate()
  ability:activate(parent)
end

function move_parent(parent, ability, targets)
  dest = targets:selected_point()
  parent:teleport_to(dest)
end
