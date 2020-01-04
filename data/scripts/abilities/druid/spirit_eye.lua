function on_activate(parent, ability)
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("1by1")
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  ability:activate(parent)
  
  local summon = game:spawn_actor_at("spirit_eye", pos.x, pos.y, parent:get_faction())
  if not summon:is_valid() then return end
  
  if parent:is_party_member() then
    summon:add_to_party(false)
	summon:set_flag("__is_summoned_party_member")
  end
  
  local effect = summon:create_effect(ability:name(), ability:duration())
  local cb = ability:create_callback(summon)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local anim = summon:create_color_anim()
  anim:set_color(anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.5))
  effect:add_color_anim(anim)
  
  effect:apply()
  
  local anim = summon:create_color_anim(1.0)
  anim:set_color(anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.5))
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  local anim = parent:create_color_anim(1.0)
  anim:set_color(anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.6),
                 anim:param(0.5, -0.5))
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