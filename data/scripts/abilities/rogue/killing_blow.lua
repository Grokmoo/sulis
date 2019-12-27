function on_activate(parent, ability)
  local stats = parent:stats()
  if not stats.attack_is_melee then
    game:say_line("You must have a melee weapon equipped.", parent)
    return
  end

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
  cb:set_after_attack_fn("do_attack")
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Dummy", "Melee", 0, 0, 0, "Raw", cb)
  
  local anim = target:create_color_anim(1.0)
  anim:set_color(anim:param(1.0, 0.0),
                 anim:param(0.2, 0.8),
                 anim:param(0.2, 0.8),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.4, -0.4),
                     anim:param(0.1, -0.1),
                     anim:param(0.1, -0.1),
                     anim:param(0.0))
  anim:activate()
end

function do_attack(parent, ability, targets)
  local target = targets:first()
  local stats = parent:stats()

  local min_dmg = stats.damage_min_0
  local max_dmg = stats.damage_max_0
  
  local target_stats = target:stats()
  local cur_hp = target_stats.current_hp
  local max_hp = target_stats.max_hp
  
  local threshold = 0.15 + 0.10 * parent:ability_level(ability)
  
  if cur_hp / max_hp < threshold then
    target:take_damage(parent, cur_hp, cur_hp, "Raw")
  else
    target:take_damage(parent, min_dmg, max_dmg, "Raw")
  end

  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-1.25), gen:param(-1.25))
  gen:set_particle_size_dist(gen:fixed_dist(2.5), gen:fixed_dist(2.5))
  gen:set_color(gen:param(1.0), gen:param(0.2), gen:param(0.2))
  gen:activate()
end
