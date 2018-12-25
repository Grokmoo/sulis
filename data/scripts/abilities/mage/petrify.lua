function on_activate(parent, ability)
  targets = parent:targets():hostile():visible_within(7)
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  target = targets:first()
  
  hit = parent:special_attack(target, "Fortitude", "Spell")
  duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  stats = parent:stats()
  
  effect = target:create_effect(ability:name(), duration)
  effect:set_tag("petrify")
  effect:add_move_disabled()
  effect:add_attack_disabled()
  effect:add_num_bonus("defense", -30 - stats.caster_level)
  effect:add_num_bonus("fortitude", -20 - stats.caster_level)
  effect:add_num_bonus("reflex", -40 - stats.caster_level)
  effect:add_num_bonus("will", -20 - stats.caster_level)
  effect:add_armor_of_kind(-8, "Crushing")
  
  anim = target:create_color_anim()
  anim:set_color(anim:param(0.4),
                 anim:param(0.3),
                 anim:param(0.3),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.3),
                     anim:param(0.2),
                     anim:param(0.2),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()
end
