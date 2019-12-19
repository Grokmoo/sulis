radius = 9.0

function on_activate(parent, ability)
  local targets = parent:targets():friendly()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_shape_object_size("9by9round")
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:create_anim("star_circle", 0.7)
  anim:set_position(anim:param(parent:x() - 3.0), anim:param(parent:y() - 4.0))
  anim:set_color(anim:param(1.0), anim:param(0.5), anim:param(0.0))
  anim:set_draw_above_entities()
  anim:set_particle_size_dist(anim:fixed_dist(8.0), anim:fixed_dist(8.0))
  anim:activate()

  local anim = parent:wait_anim(0.5)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_fire_surface")
  anim:set_completion_callback(cb)
  anim:activate()
  
  ability:activate(parent)
end

function create_fire_surface(parent, ability, targets)
  local points = targets:random_affected_points(0.7)
  local surf = parent:create_surface("Fire", points, 3)
  surf:set_squares_to_fire_on_moved(3)
  
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surf:add_callback(cb)
  
  local gen = parent:create_particle_generator("fire_particle")
  gen:set_alpha(gen:param(0.75))
  gen:set_gen_rate(gen:param(30.0))
  gen:set_position(gen:param(0.0), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.1, 0.1)),
								 gen:dist_param(gen:uniform_dist(0.0, 0.5), gen:uniform_dist(-2.0, -3.0)))
  gen:set_draw_above_entities()
  surf:add_anim(gen)
  
  local below = parent:create_anim("particles/circle16")
  below:set_draw_below_entities()
  below:set_position(below:param(-0.25), below:param(-0.25))
  below:set_particle_size_dist(below:fixed_dist(1.5), below:fixed_dist(1.5))
  below:set_color(below:param(0.8), below:param(0.5), below:param(0.0), below:param(0.2))
  surf:add_anim(below)
  
  surf:apply()
  
  local targets = targets:to_table()
  for i = 1, #targets do
    create_aegis_effect(parent, ability, targets[i])
  end
end

function create_aegis_effect(parent, ability, target)
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:add_resistance(100, "Fire")
  
  local stats = parent:stats()
  local min_dmg = 2 + stats.caster_level / 4 + stats.intellect_bonus / 6
  local max_dmg = 4 + stats.intellect_bonus / 3 + stats.caster_level / 2
  effect:add_damage_of_kind(min_dmg, max_dmg, "Fire")
  
  local anim = target:create_anim("spin_slash")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-2.0), anim:param(-3.0))
  anim:set_draw_above_entities()
  anim:set_particle_size_dist(anim:fixed_dist(4.0), anim:fixed_dist(4.0))
  effect:add_anim(anim)
  
  local cb = ability:create_callback(target)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  target:add_ability("flaming_bolt")
  
  effect:apply()
end

-- Aegis Effect on remove

function on_removed(parent)
  parent:remove_ability("flaming_bolt")
end

-- Surface methods

function on_moved(parent, ability, targets)
  local target = targets:first()
  target:take_damage(parent, 3, 6, "Fire", 2)
end

function on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:take_damage(parent, 3, 6, "Fire", 2)
  end
end
