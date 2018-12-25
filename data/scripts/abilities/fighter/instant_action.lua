function on_activate(parent, ability)
  ability:activate(parent)

  amount = 2 * game:ap_display_factor()
  parent:change_overflow_ap(-amount)
  parent:add_ap(amount)
  
  gen = parent:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-1.25), gen:param(-1.25))
  gen:set_particle_size_dist(gen:fixed_dist(2.5), gen:fixed_dist(2.5))
  gen:activate()
end
