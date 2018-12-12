function on_activate(parent, ability)
  targets = parent:targets():friendly():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  target = targets:first()
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("ward")
  
  cb = ability:create_callback(parent)
  cb:set_on_effect_applied_fn("on_effect_applied")
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  gen = target:create_anim("ring")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-1.5), gen:param(-1.2))
  gen:set_particle_size_dist(gen:fixed_dist(3.0), gen:fixed_dist(3.0))
  gen:set_draw_below_entities()
  effect:add_anim(gen)
  effect:apply()
end

function on_effect_applied(parent, ability, targets, effect)
  -- game:log(effect:name() .. " applied to " .. parent:name() .. " with tag " .. effect:tag())
  
  local matching_tags = {
    sleep = true,
    nauseate = true,
    sundered_armor = true,
    stuck = true,
    vulnerable = true,
    cripple = true,
    blind = true,
    polymorph = true,
    slow = true,
    weaken = true,
    damage = true,
    petrify = true,
	dazzle = true,
	disease = true,
	rupture = true,
  }
  
  if matching_tags[effect:tag()] ~= nil then
    game:say_line("Ward!", parent)
    effect:mark_for_removal()
	
	uses = parent:get_num_flag("__ward_uses")
	uses = uses + 1
	parent:add_num_flag("__ward_uses", uses)
	
	if uses >= 2 then
	  parent:remove_effects_with_tag("ward")
	end
  end
end

function on_removed(parent)
  parent:clear_flag("__ward_uses")
end