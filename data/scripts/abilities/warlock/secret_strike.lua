function on_activate(parent, ability)
  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_attackable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_before_attack_fn("create_parent_effect")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)

  game:play_sfx("sfx/swish-9")
end

function create_parent_effect(parent, ability, targets)
  local target = targets:first()
  local stats = parent:stats()

  local effect = parent:create_effect(ability:name(), 0)
  
  local stats = parent:stats()
  local amount = 20 + stats.caster_level
  
  effect:add_num_bonus("melee_accuracy", amount)
  effect:add_num_bonus("ranged_accuracy", amount)
  effect:add_num_bonus("crit_chance", 10)
  effect:apply()
  
  local duration = 0.89
  local anim = parent:create_anim("teleport", duration)
  anim:set_position(anim:param(parent:center_x() - 0.5),
                    anim:param(parent:center_y() - 1.75))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(3.0))
  anim:set_color(anim:param(0.0), anim:param(0.0), anim:param(0.0), anim:param(0.9))
  anim:activate()
end
