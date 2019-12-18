function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(4.0)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_reachable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  
  local anim = target:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1.0),
                     anim:param(0.0),
                     anim:param(0.0),
                     anim:param(0.0))
  anim:activate()
  
  local stats = parent:stats()
  local min_dmg = 10 + stats.caster_level / 2 + stats.intellect_bonus / 4
  local max_dmg = 20 + stats.intellect_bonus / 2 + stats.caster_level
  local hit = parent:special_attack(target, "Fortitude", "Spell", min_dmg, max_dmg, 0, "Raw")
  
  local damage = hit:total_damage()
  
  if damage <= 0 then
    return
  end
  
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(0.0, 1.0),
                     anim:param(0.0),
                     anim:param(0.0),
                     anim:param(0.0))
  anim:activate()
  
  parent:heal_damage(damage)
end
