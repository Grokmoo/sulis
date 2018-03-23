function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("defense", 20)
  effect:add_num_bonus("reflex", 10)
  effect:add_num_bonus("fortitude", 10)
  effect:add_num_bonus("will", 10)

  gen = parent:create_particle_generator("firebolt")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.25), gen:param(-0.25))
  gen:set_gen_rate(gen:param(10.0))
  gen:set_initial_gen(5.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_x_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5)))
  gen:set_particle_y_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))

  effect:apply(gen)

  ability:activate(parent)
end
