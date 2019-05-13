function on_activate(parent, ability)
  local targets = parent:targets():friendly():visible_within(8)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(8.0)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  
  local effect = target:create_effect(ability:name(), ability:duration())
  
  local min_dmg = 3 + stats.caster_level / 4 + stats.intellect_bonus / 8
  local max_dmg = 6 + stats.caster_level / 2 + stats.intellect_bonus / 4
  effect:add_damage_of_kind(min_dmg, max_dmg, "Acid")
  
  local anim = target:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_initial_gen(8.0)
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.2))
  anim:set_gen_rate(anim:param(20.0))
  anim:set_position(anim:param(-1.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
  
  ability:activate(parent)
end
