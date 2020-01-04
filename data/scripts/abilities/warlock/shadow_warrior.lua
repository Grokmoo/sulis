function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:set_shape_circle(7.0)
  targeter:add_all_effectable(targets)
  targeter:allow_affected_points_impass(false)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  ability:activate(parent)
  
  local stats = parent:stats()
  local amount = math.floor(1 + stats.caster_level / 5)
  local points = targets:affected_points()
  
  for i=1,amount do
    gen_summon(parent, ability, points)
  end
end

function gen_summon(parent, ability, points)
  local summon = try_find_position(points)
  if summon == nil then return end
  
  if parent:is_party_member() then
    summon:add_to_party(false)
    summon:set_flag("__is_summoned_party_member")
  end
  
  local effect = summon:create_effect(ability:name(), ability:duration())
  effect:add_abilities_disabled()
  effect:add_attack_disabled()
  
  local cb = ability:create_callback(summon)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)

  local anim = summon:create_color_anim()
  anim:set_color(anim:param(0.01), anim:param(0.01), anim:param(0.01), anim:param(1.0))
  effect:add_color_anim(anim)
  
  effect:apply()
end

function try_find_position(points)
  -- try to find a place for the summon, at most 20 attempts
  for i=1,20 do
    -- generate random point
	local pos = points[math.random(#points)]
	
	local summon = game:spawn_actor_at("shadow_warrior", pos.x, pos.y, parent:get_faction())
    if summon:is_valid() then return summon end
  end
  
  return nil
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