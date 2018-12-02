radius = 7.0

function on_activate(parent, ability)
  targets = parent:targets():friendly()
  
  targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  
  position = targets:selected_point()
  
  gen = parent:create_particle_generator("wind_particle", 0.6)
  gen:set_initial_gen(1000.0)
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  speed = radius / 0.5
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.5), gen:param(1.0, -2.0))
  
  targets = targets:to_table()
  for i = 1, #targets do
    cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("apply_effect")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
end

function apply_effect(parent, ability, targets)
  stats = parent:stats()
  amount = 10 + stats.caster_level / 2 + stats.wisdom_bonus / 4

  target = targets:first()
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("defense", amount / 2)
  effect:add_num_bonus("fortitude", amount)
  effect:add_num_bonus("reflex", amount)
  effect:add_num_bonus("will", amount)

  anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-2.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.5), anim:param(0.5))
  effect:add_anim(anim)
  effect:apply()
end
