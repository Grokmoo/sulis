function on_activate(parent, ability)
  local stats = parent:stats()
  if not stats.attack_is_melee then
    game:say_line("You must have a melee weapon equipped.", parent)
    return
  end

  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_reachable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_grapple_effect")
  
  ability:activate(parent)
  
  local accuracy = 10 - 5 * target:width()
  if parent:ability_level(ability) > 1 then
    accuracy = accuracy + 20
  end
  
  local effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("melee_accuracy", accuracy)
  effect:apply()
  
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_grapple_effect(parent, ability, targets, hit)
  local target = targets:first()
  
  if hit:is_miss() then
    return
  end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  
  if hit:is_graze() then
    effect:add_move_disabled()
  else
    effect:add_move_disabled()
    effect:add_attack_disabled()
  end
  
  local gen = target:create_anim("imprison")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.74), gen:param(-1.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:set_color(gen:param(1.0), gen:param(0.2), gen:param(0.0))
  effect:add_anim(gen)
  effect:apply()
end
