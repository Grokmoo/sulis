function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
    game:say_line("You must have a melee weapon equipped.", parent)
    return
  end

  targets = parent:targets():hostile():attackable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_feint_effect")
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Will", "Melee", 0, 0, 0, "Raw", cb)
end

function create_feint_effect(parent, ability, targets, hit)
  target = targets:first()

  if hit:is_miss() then return end

  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("vulnerable")
  stats = parent:stats()
  
  if hit:is_graze() then
    effect:add_num_bonus("defense", -10 - stats.level)
  elseif hit:is_hit() then
    effect:add_num_bonus("defense", -20 - stats.level * 1.5)
  elseif hit:is_crit() then
    effect:add_num_bonus("defense", -30 - stats.level * 2)
  end

  anim = target:create_particle_generator("arrow_down")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  effect:add_anim(anim)
  
  effect:apply()
end
