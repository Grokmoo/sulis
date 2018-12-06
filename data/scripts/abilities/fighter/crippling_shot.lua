function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_ranged then
    game:say_line("You must have a ranged weapon equipped.", parent)
    return
  end

  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_cripple_effect")
  cb:set_before_attack_fn("create_parent_penalty")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_parent_penalty(parent, ability, targets)
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("graze_multiplier", -0.25)
  effect:add_num_bonus("hit_multiplier", -0.5)
  effect:add_num_bonus("crit_multiplier", -1.0)
  effect:apply()
end

function create_cripple_effect(parent, ability, targets, hit)
  target = targets:first()

  if hit:is_miss() then return end
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("cripple")
  stats = parent:stats()
  
  if hit:is_graze() then
    effect:add_num_bonus("movement_rate", -0.5 - stats.level / 60)
  elseif hit:is_hit() then
    effect:add_num_bonus("movement_rate", -0.75 - stats.level / 40)
  elseif hit:is_crit() then
    effect:add_num_bonus("movement_rate", -1.0 - stats.level / 40)
  end
  
  anim = target:create_particle_generator("particles/circle4")
  anim:set_initial_gen(10.0)
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  anim:set_gen_rate(anim:param(10.0))
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(0.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
end

