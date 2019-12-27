function on_activate(parent, ability)
  local targets = parent:targets():hostile():touchable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_touchable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  
  local hit = parent:special_attack(target, "Will", "Spell")
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
  effect:set_tag("fear")
  effect:add_attack_disabled()
  effect:add_num_bonus("will", -20)
  
  local gen = target:create_anim("terror")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-3.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  effect:add_anim(gen)
  effect:apply()
end
