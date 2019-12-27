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
  
  local hit = parent:special_attack(target, "Fortitude", "Melee")
  local amount = -4 * game:ap_display_factor()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  target:change_overflow_ap(amount)
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("sleep")
  effect:add_num_bonus("ap", amount)
  effect:add_move_disabled()
  effect:add_attack_disabled()
  
  local cb = ability:create_callback(parent)
  cb:set_on_damaged_fn("on_damaged")
  effect:add_callback(cb)
  
  local anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(0.5), anim:param(0.5), anim:param(0.5), anim:param(0.5))
  effect:add_anim(anim)
  effect:apply()
  
  ability:activate(parent)
end

function on_damaged(parent, ability, targets)
  local target = targets:first()
  local parent = targets:parent() -- parent passed to the func is actually the caster here
  parent:remove_effects_with_tag("sleep")
end
