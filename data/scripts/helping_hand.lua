function on_activate(parent, ability)
  targets = parent:targets():friendly():reachable():without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  target = targets:first()
  
  ap = target:get_overflow_ap()
  new_ap = math.min(0, ap + 4000)
  if new_ap ~= ap then
    target:change_overflow_ap(new_ap - ap)
  end
  
  target:remove_effects_with_tag("slow")
  
  anim = target:create_particle_generator("sparkle", 1.0)
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.8), anim:fixed_dist(0.8))
  anim:set_gen_rate(anim:param(5.0, -5.0))
  anim:set_initial_gen(3.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.5, -1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(0.0), anim:param(0.5), anim:param(1.0, -1.0), anim:param(1.0, -1.0))
  anim:activate()
end
