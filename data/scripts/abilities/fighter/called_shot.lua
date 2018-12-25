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
  cb:set_before_attack_fn("create_parent_bonus")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_parent_bonus(parent, ability, targets)
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("crit_chance", 100)
  effect:apply()
end