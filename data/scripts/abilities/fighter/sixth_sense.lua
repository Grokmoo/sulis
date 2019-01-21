function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  
  local stats = parent:stats()
  effect:add_num_bonus("concealment_ignore", 40 + stats.level + stats.perception_bonus / 2)
  effect:add_attribute_bonus("Perception", 2 + stats.level / 2)
  effect:add_flanked_immunity()
  effect:add_sneak_attack_immunity()

  local gen = parent:create_anim("eye")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-3.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_draw_above_entities()
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end
