function on_activate(parent, ability)
  local targets = parent:targets():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:set_callback_fn("on_target")
  targeter:activate()
end

function on_target(parent, ability, targets)
  local target = targets:first()
  
  local stats = parent:stats()
  local range = 7 + stats.caster_level / 2

  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(range + 2)
  targeter:set_free_select(range + 2)
  targeter:set_free_select_must_be_passable(parent:size_str())
  targeter:set_shape_object_size(parent:size_str())
  targeter:set_callback_fn("on_position")
  targeter:set_callback_custom_target(target)
  targeter:activate()
end

function on_position(parent, ability, targets, custom_target)
  local pos = targets:selected_point()
  
  local duration = 0.89
  local anim = parent:create_anim("teleport", duration)
  anim:set_position(anim:param(custom_target:center_x() - 0.5),
                    anim:param(custom_target:center_y() - 1.75))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(3.0))
  
  local cb = ability:create_callback(parent)
  cb:add_target(custom_target)
  cb:add_selected_point(pos)
  cb:set_on_anim_update_fn("move_target")
  anim:add_callback(cb, duration - 0.2)
  
  anim:activate()
  ability:activate(parent)
  
  game:play_sfx("sfx/teleport")
end

function move_target(parent, ability, targets)
  local target = targets:first()
  local pos = targets:selected_point()

  target:teleport_to(pos)
end
