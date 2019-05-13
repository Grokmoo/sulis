function on_activate(parent, ability)
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(15.0)
  targeter:set_free_select(15.0)
  targeter:set_shape_object_size("1by1")
  targeter:set_callback_fn("on_target")
  targeter:activate()
end

function on_target(parent, ability, targets)
  local selected_point = targets:selected_point()
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(15.0)
  targeter:set_free_select(15.0)
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_line("1by1", selected_point.x, selected_point.y, 8)
  targeter:invis_blocks_affected_points(true)
  targeter:impass_blocks_affected_points(true)
  targeter:set_callback_fn("on_position")
  targeter:activate()
end

function on_position(parent, ability, targets)
  local points = targets:affected_points()
  for i = 1, #points do
    local point = points[i]
	local summon = game:spawn_actor_at("stone", point.x, point.y, "Neutral")

	if summon:is_valid() then
	  local effect = summon:create_effect(ability:name(), ability:duration())
	
      local cb = ability:create_callback(summon)
      cb:set_on_removed_fn("on_removed")
      effect:add_callback(cb)
      effect:apply()
      
      local anim = summon:create_color_anim(1.0)
      anim:set_color_sec(anim:param(1.0, -1,0),
                        anim:param(1.0, -1,0),
                         anim:param(1.0, -1,0),
                         anim:param(0.0))
      anim:activate()
	end
  end

  ability:activate(parent)
end

function on_removed(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  local anim = parent:create_color_anim(1.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(1.0, -1.0))
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:set_completion_callback(cb)
  anim:activate()
end

function on_remove_complete(parent, ability)
  parent:remove()
end