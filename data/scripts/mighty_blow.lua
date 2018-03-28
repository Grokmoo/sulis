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

function on_target_select(parent, ability, targets, selected_point)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:register_fn("before_attack")
  
  -- Remove an additional point of AP beyond the standard attack
  parent:remove_ap(10)
  -- ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function before_attack(parent, ability, targets)
  target = targets:first()
  stats = parent:stats()

  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", 25)
  effect:add_damage(10, 15)
  effect:apply()
  
  gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
