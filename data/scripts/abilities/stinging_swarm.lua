function on_activate(parent, ability)
  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(12.0)
  targeter:set_shape_circle(5.0)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  points = targets:affected_points()
  surface = parent:create_surface(ability:name(), points, ability:duration())
  
  stats = parent:stats()
  bonus = 10 + stats.caster_level / 2 + stats.wisdom_bonus / 4
  surface:add_num_bonus("defense", -bonus)
  surface:add_num_bonus("reflex", -bonus)
  surface:add_num_bonus("melee_accuracy", -bonus)
  surface:add_num_bonus("ranged_accuracy", -bonus)
  surface:add_num_bonus("spell_accuracy", -bonus)
  
  surface:set_squares_to_fire_on_moved(3)
  cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surface:add_callback(cb)
  
  s_anim = parent:create_particle_generator("particles/circle4")
  s_anim:set_position(s_anim:param(0.0), s_anim:param(0.0))
  s_anim:set_color(s_anim:param(0.0), s_anim:param(0.0), s_anim:param(0.0), s_anim:param(1.0))
  s_anim:set_gen_rate(s_anim:param(20.0))
  s_anim:set_particle_size_dist(s_anim:fixed_dist(0.1), s_anim:fixed_dist(0.1))
  s_anim:set_particle_duration_dist(s_anim:fixed_dist(1.0))
  s_anim:set_particle_position_dist(s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-1.0, 1.0)),
                                    s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-1.0, 1.0)))
  s_anim:set_draw_above_entities()
  surface:add_anim(s_anim)
  surface:apply()
  
  ability:activate(parent)
end

function on_moved(parent, ability, targets)
  target = targets:first()
  target:take_damage(parent, 2, 4, "Piercing", 18)
end

function on_round_elapsed(parent, ability, targets)
  targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:take_damage(parent, 2, 4, "Piercing", 18)
  end
end
