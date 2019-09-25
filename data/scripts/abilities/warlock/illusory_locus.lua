function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  
  local stats = parent:stats()
  local anim = parent:create_color_anim()
  anim:set_color(anim:param(1.0),
                 anim:param(1.0),
                 anim:param(1.0),
                 anim:param(0.4))
  effect:add_color_anim(anim)
  
  local w = parent:width()
  local h = parent:height()
  
  local anim = parent:create_particle_generator("teleport")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(0.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.333), anim:fixed_dist(2.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-1.0 - w, 1.0)),
                                  anim:dist_param(anim:uniform_dist(-1.0 - h, 1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.5))
  anim:set_color(anim:param(0.5), anim:param(0.5), anim:param(0.5), anim:param(1.0))
  effect:add_anim(anim)
  
  local cb = ability:create_callback(parent)
  cb:set_before_defense_fn("illusion_check")
  effect:add_callback(cb)

  effect:apply()
  ability:activate(parent)
end

function illusion_check(parent, ability, targets)
  -- 25% chance to miss
  if math.random() < 0.75 then
    return
  end

  local target = targets:first()
  
  local effect = target:create_effect(ability:name(), 0)
  effect:add_num_bonus("melee_accuracy", -100)
  effect:add_num_bonus("ranged_accuracy", -100)
  effect:add_num_bonus("spell_accuracy", -100)
  effect:apply()
  
  local gen = parent:create_anim("wind_particle", 0.5)
  gen:set_moves_with_parent()
  gen:set_initial_gen(100)
  gen:set_position(gen:param(0.0), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = 5.0
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(1.0), gen:param(1.0), gen:param(1.0, -2.0))
  gen:activate()
end