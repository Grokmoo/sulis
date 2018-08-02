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
  
  parent:special_attack(target, "Reflex", "Ranged", 20, 30, 0, "Piercing")
end