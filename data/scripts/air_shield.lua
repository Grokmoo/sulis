function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_armor_of_kind(5, "Electrical")
  effect:add_armor_of_kind(5, "Sonic")

  cb = ability:create_callback(parent)
  cb:set_before_defense_fn("before_defense")
  effect:add_callback(cb)

  gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(70.0))
  gen:set_position(gen:param(-0.25), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(0.0, 0.2), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end

function before_defense(parent, ability, targets)
  target = targets:first()

  stats = target:stats()
  if not stats.attack_is_ranged then
    return
  end
  
  -- 50% chance to block
  if math.random() < 0.5 then
    return
  end
  
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("defense", 100)
  effect:apply()
  
  gen = parent:create_anim("wind_particle", 0.5)
  gen:set_moves_with_parent()
  gen:set_initial_gen(100)
  gen:set_position(gen:param(0.0), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  speed = 5.0
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(1.0), gen:param(1.0), gen:param(1.0, -2.0))
  gen:activate()
end
