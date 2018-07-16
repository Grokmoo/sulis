function on_activate(parent, ability)
  targets = parent:targets():friendly():reachable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  effect = target:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("movement_rate", 0.5)
  effect:add_num_bonus("defense", 5)
  effect:add_num_bonus("reflex", 5)

  gen = target:create_particle_generator("wind_particle", duration)
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(30.0))
  gen:set_position(gen:param(-0.25), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.25), gen:fixed_dist(0.25))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(0.0, 0.2), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  effect:apply(gen)

  ability:activate(parent)
end