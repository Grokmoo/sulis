function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_visible()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  game:play_sfx("sfx/disenchant")
  
  local stats = parent:stats()
  local amount = 10 + stats.caster_level / 2 + stats.intellect_bonus / 4
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("assay")
  effect:add_num_bonus("defense", -amount)
  effect:add_num_bonus("fortitude", -amount)
  effect:add_num_bonus("reflex", -amount)
  effect:add_num_bonus("will", -amount)
  
  local gen = target:create_particle_generator("arrow_down")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(0.0), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_gen_rate(gen:param(6.0))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.7, 0.7), gen:uniform_dist(-0.1, 0.1)),
                                 gen:dist_param(gen:fixed_dist(0.0), gen:uniform_dist(1.0, 1.5)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.75))
  gen:set_color(gen:param(1.0), gen:param(0.2), gen:param(0.1))
  effect:add_anim(gen)
  effect:apply()
end