function on_activate(parent, ability)
  targets = parent:targets():visible_within(10)
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:set_callback_fn("on_target")
  targeter:activate()
end

function on_target(parent, ability, targets)
  target = targets:first()

  targeter = parent:create_targeter(ability)
  targeter:set_free_select(12.0)
  targeter:set_free_select_must_be_passable(parent:size_str())
  targeter:set_shape_object_size(parent:size_str())
  targeter:set_callback_fn("on_position")
  targeter:set_callback_custom_target(target)
  targeter:activate()
end

function on_position(parent, ability, targets, custom_target)
  pos = targets:selected_point()
  
  duration = 0.89
  anim = parent:create_anim("teleport", duration)
  anim:set_position(anim:param(custom_target:center_x() - 1.0),
                    anim:param(custom_target:center_y() - 2.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(3.0))
  
  cb = ability:create_callback(parent)
  cb:add_target(custom_target)
  cb:add_selected_point(pos)
  cb:set_on_anim_update_fn("move_target")
  anim:add_callback(cb, duration - 0.2)
  
  anim:activate()
  ability:activate(parent)
end

function move_target(parent, ability, targets)
  target = targets:first()
  pos = targets:selected_point()

  target:teleport_to(pos)
end
