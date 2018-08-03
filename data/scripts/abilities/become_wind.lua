function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("movement_rate", 0.8)
  effect:add_num_bonus("reflex", 10)
  effect:add_attribute_bonus("Dexterity", 4)

  gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(70.0))
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-1.0, 1.0), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(-1.0, 1.0), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end
