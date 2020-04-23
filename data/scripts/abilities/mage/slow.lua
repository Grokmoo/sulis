function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_visible()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local stats = parent:stats()
  ability:activate(parent)

  local target = targets:first()
  
  local hit = parent:special_attack(target, "Will", "Spell")
  local amount = -(2 + stats.intellect_bonus / 20) * game:ap_display_factor()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("slow")
  effect:add_num_bonus("ap", amount)
  effect:add_num_bonus("move_anim_rate", -0.3)
  
  local gen = target:create_anim("slow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()
  
  game:play_sfx("sfx/echo01")
end
