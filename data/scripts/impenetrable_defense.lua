function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("defense", 25)
  effect:add_num_bonus("reflex", 10)
  effect:add_num_bonus("fortitude", 10)
  effect:add_num_bonus("will", 10)

  gen = parent:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-01.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  effect:apply(gen)

  ability:activate(parent)
end
