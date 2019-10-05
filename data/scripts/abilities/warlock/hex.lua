function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(7)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(7)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local target = targets:first()
  
  local hit = parent:special_attack(target, "Fortitude", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("hex")
  effect:add_abilities_disabled()

  local gen = target:create_anim("slow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0), gen:param(0.8))
  effect:add_anim(gen)
  effect:apply()
end
