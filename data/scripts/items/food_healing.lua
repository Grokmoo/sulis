function on_activate(parent, item)
  parent:heal_damage(5)

  anim = parent:create_particle_generator("heal", 1.0)
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.5, -1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:activate()
  
  item:activate(parent)
end