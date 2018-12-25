function on_activate(parent, ability)
  if game:num_effects_with_tag("trap") > 4 then
    game:say_line("Maximum number of traps set.", parent)
    return
  end

  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(1.0)
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("1by1")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  points = targets:affected_points()

  surf = parent:create_surface(ability:name(), points)
  surf:set_tag("trap")
  surf:set_squares_to_fire_on_moved(1)
  
  cb = ability:create_callback(parent)
  cb:set_on_moved_in_surface_fn("on_entered")
  surf:add_callback(cb)
  
  anim = parent:create_anim("particles/spike_trap_set")
  anim:set_position(anim:param(0.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(2.0))
  anim:set_draw_below_entities()
  surf:add_anim(anim)
  
  surf:apply()
  ability:activate(parent)
end

function on_entered(parent, ability, targets)
  -- only fire on hostiles
  if targets:hostile():is_empty() then return end
  
  targets:surface():mark_for_removal()
  
  target = targets:first()
  points = targets:affected_points()
  point = points[1]
  
  anim = target:create_anim("particles/spike_trap_fired", 0.5)
  anim:set_position(anim:param(point.x), anim:param(point.y - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(2.0))
  anim:set_draw_above_entities()
  anim:activate()
  
  stats = parent:stats()
  min_dmg = 18 + stats.level / 2 + stats.intellect_bonus / 4
  max_dmg = 28 + stats.level + stats.intellect_bonus / 2
  
  hit = parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 0, "Piercing")
  
  if not target:is_dead() and parent:ability_level(ability) > 1 then
    effect = target:create_effect(ability:name(), 2)
    
	if hit:is_miss() then return end
	
    if hit:is_graze() then
      effect:add_num_bonus("movement_rate", -0.25)
    elseif hit:is_hit() then
      effect:add_num_bonus("movement_rate", -0.5)
    elseif hit:is_crit() then
      effect:add_num_bonus("movement_rate", -0.75)
    end
    
    anim = target:create_particle_generator("particles/circle4")
    anim:set_initial_gen(10.0)
    anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
    anim:set_gen_rate(anim:param(10.0))
    anim:set_moves_with_parent()
    anim:set_position(anim:param(0.0), anim:param(0.0))
    anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
    anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
      anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
    anim:set_particle_duration_dist(anim:fixed_dist(0.3))
    effect:add_anim(anim)
    effect:apply()
  end
end