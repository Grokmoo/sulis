function on_activate(parent, ability)
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(12.0)
  targeter:set_free_select_must_be_passable("3by3")
  targeter:set_shape_object_size("3by3")
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  pos = targets:selected_point()
  ability:activate(parent)
  
  summon = game:spawn_actor_at("wind_elemental", pos.x, pos.y, "Friendly")
  if not summon:is_valid() then return end
  
  summon:add_to_party(false)
  
  levels = parent:stats().caster_level
  if levels > 3 then
    summon:add_levels("mage", levels - 3)
  end
  
  if parent:ability_level(ability) > 1 then
    summon:add_ability("wind_gust")
	summon:add_ability("shock")
  end
  
  effect = summon:create_effect(ability:name(), ability:duration())
  cb = ability:create_callback(summon)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  effect:apply()
  
  anim = summon:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  anim = parent:create_color_anim(1.0)
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