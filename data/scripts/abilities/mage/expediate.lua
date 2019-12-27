function on_activate(parent, ability)
  local targets = parent:targets():friendly():touchable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_touchable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("haste")
  effect:add_num_bonus("movement_rate", 0.5 + stats.intellect_bonus * 0.01 + stats.caster_level * 0.02)
  effect:add_num_bonus("defense", 5 + stats.intellect_bonus / 4 + stats.caster_level / 2)
  effect:add_num_bonus("reflex", 5 + stats.intellect_bonus / 4 + stats.caster_level / 2)

  local gen = target:create_particle_generator("wind_particle")
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(30.0))
  gen:set_position(gen:param(-0.25), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.25), gen:fixed_dist(0.25))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(0.0, 0.2), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end
