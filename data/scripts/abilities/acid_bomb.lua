function on_activate(parent, ability)
  targets = parent:targets():hostile():visible_within(8)
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  target = targets:first()
  
  hit = parent:special_attack(target, "Reflex", "Spell")
  duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration * 3 / 2
  end
  
  target:take_damage(4, 8, "Acid")
  
  effect = target:create_effect(ability:name(), duration)
  effect:add_num_bonus("melee_accuracy", -10)
  effect:add_num_bonus("ranged_accuracy", -10)
  effect:add_num_bonus("spell_accuracy", -10)
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_damage")
  effect:add_callback(cb)
  
  anim = target:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_initial_gen(10.0)
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.2))
  anim:set_gen_rate(anim:param(20.0))
  anim:set_position(anim:param(0.0), anim:param(-0.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
end

function apply_damage(parent, ability, targets)
  target = targets:first()
  
  target:take_damage(4, 8, "Acid")
end
