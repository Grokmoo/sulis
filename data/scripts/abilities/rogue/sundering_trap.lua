function on_activate(parent, ability)
  if game:num_effects_with_tag("trap") > 4 then
    game:say_line("Maximum number of traps set.", parent)
    return
  end

  local targets = parent:targets()
  local radius = parent:stats().touch_distance
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(radius)
  targeter:set_free_select(radius)
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("1by1")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()

  local surf = parent:create_surface(ability:name(), points)
  surf:set_tag("trap")
  surf:set_squares_to_fire_on_moved(1)
  
  local cb = ability:create_callback(parent)
  cb:set_on_moved_in_surface_fn("on_entered")
  surf:add_callback(cb)
  
  local anim = parent:create_anim("particles/spike_trap_set")
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
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
  
  local target = targets:first()
  local points = targets:affected_points()
  local point = points[1]
  
  local anim = target:create_anim("particles/spike_trap_fired", 0.5)
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  anim:set_position(anim:param(point.x), anim:param(point.y - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(2.0))
  anim:set_draw_above_entities()
  anim:activate()
  
  local hit = parent:special_attack(target, "Reflex", "Ranged", 10, 15, 5, "Piercing")
  
  local effect = target:create_effect(ability:name(), 2)
  effect:set_tag("sundered_armor")
  
  if hit:is_miss() then return end

  local stats = parent:stats()
  local amount = 8 + stats.level / 2
  if parent:has_ability("mechanical_mastery") then
    amount = amount + 3
  end

  if hit:is_graze() then
    effect:add_num_bonus("armor", -amount / 1.5)
  elseif hit:is_hit() then
    effect:add_num_bonus("armor", -amount)
  elseif hit:is_crit() then
    effect:add_num_bonus("armor", -(amount * 1.5))
  end
  
  local gen = target:create_particle_generator("shield")
  gen:set_initial_gen(3.0)
  gen:set_gen_rate(gen:param(3.0))
  gen:set_moves_with_parent()
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0))
  gen:set_position(gen:param(-0.5), gen:param(-1.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:fixed_dist(1.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.95))
  effect:add_anim(gen)
  
  effect:apply()
end