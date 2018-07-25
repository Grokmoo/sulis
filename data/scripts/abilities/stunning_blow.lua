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
  cb:set_after_attack_fn("create_stun_effect")
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_stun_effect(parent, ability, targets, hit)
  target = targets:first()
  
  if hit:is_graze() then
    target:change_overflow_ap(-2000)
  elseif hit:is_hit() then
    target:change_overflow_ap(-4000)
  elseif hit:is_crit() then
    target:change_overflow_ap(-6000)
  end
  
  gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
