function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(10)
  
  local targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local stats = parent:stats()
  local target = targets:first()
  
  local hit = parent:special_attack(target, "Will", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration * 3 / 2
  end
  
  local amount = -20 - stats.caster_level - stats.intellect_bonus / 2
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("dazzle")
  effect:add_num_bonus("melee_accuracy", amount)
  effect:add_num_bonus("ranged_accuracy", amount)
  effect:add_num_bonus("spell_accuracy", amount)
  
  local anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_initial_gen(2.0)
  anim:set_color(anim:param(1.0), anim:param(0.5), anim:param(0.0))
  anim:set_gen_rate(anim:param(8.0))
  anim:set_position(anim:param(-0.5), anim:param(-2.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-0.1, 0.1)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(1.0, 2.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  effect:add_anim(anim)
  effect:apply()
end

