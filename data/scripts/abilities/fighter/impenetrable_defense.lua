function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  
  stats = parent:stats()
  amount = 10 + stats.level / 2
  
  effect:add_num_bonus("defense", amount * 3)
  effect:add_num_bonus("reflex", amount)
  effect:add_num_bonus("fortitude", amount)
  effect:add_num_bonus("will", amount)

  gen = parent:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-01.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end
