function on_activate(parent, ability)
  local targets = parent:targets():friendly():visible_within(12)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(12.0)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local target = targets:first()

  target:remove_effects_with_tag("slow")
  target:remove_effects_with_tag("nauseate")
  target:remove_effects_with_tag("cripple")
  target:remove_effects_with_tag("blind")
  target:remove_effects_with_tag("disease")
  target:remove_effects_with_tag("rupture")
  
  local anim = target:create_particle_generator("particles/circle20", 2.0)
  anim:set_position(anim:param(target:x() + 0.5), anim:param(target:y()))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(0.0))
  anim:set_initial_gen(100.0)
  anim:set_particle_position_dist(anim:dist_param(anim:angular_dist(0.0, 2 * math.pi, 0.0, 1.0)))
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(0.0, 0.5, -0.5))
  anim:set_particle_duration_dist(anim:fixed_dist(2.0))
  anim:activate()
end
