function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
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
  cb:set_after_attack_fn("create_grapple_effect")
  
  ability:activate(parent)
  
  accuracy = 10 - 5 * target:width()
  if parent:ability_level(ability) > 1 then
    accuracy = accuracy + 20
  end
  
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", accuracy)
  effect:apply()
  
  parent:anim_special_attack(target, "Fortitude", 0, 0, 0, "Raw", cb)
end

function create_grapple_effect(parent, ability, targets, hit)
  target = targets:first()
  
  if hit:is_miss() then
    return
  end
  
  effect = target:create_effect(ability:name(), ability:duration())
  
  if hit:is_graze() then
    effect:add_move_disabled()
  else
    effect:add_move_disabled()
    effect:add_attack_disabled()
  end
  
  gen = target:create_anim("imprison")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.74), gen:param(-1.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:set_color(gen:param(1.0), gen:param(0.2), gen:param(0.0))
  effect:apply(gen)
end
