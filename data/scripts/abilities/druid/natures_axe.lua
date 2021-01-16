function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:set_shape_object_size("9by9round")
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local position = targets:selected_point()
  
  local anim = parent:create_particle_generator("particles/circle20", 2.0)
  anim:set_position(anim:param(position.x - 1.0), anim:param(position.y - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.5), anim:fixed_dist(2.5))
  anim:set_gen_rate(anim:param(0.0))
  anim:set_initial_gen(500.0)
  anim:set_particle_position_dist(anim:dist_param(anim:angular_dist(0.0, 2 * math.pi, 0.0, 4.0)))
  anim:set_color(anim:param(0.2), anim:param(0.5), anim:param(0.2), anim:param(0.0, 0.5, -0.5))
  anim:set_particle_duration_dist(anim:fixed_dist(2.0))
  anim:activate()
  
  local targets = targets:to_table()
  for i = 1, #targets do
    attack_target(parent, ability, targets[i])
  end

  ability:activate(parent)
  game:play_sfx("sfx/curse4")
end

function attack_target(parent, ability, target)
  local stats = parent:stats()
  
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
  
  local stats = parent:stats()
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("weaken")
  effect:add_num_bonus("armor", -5 - stats.caster_level / 2 - stats.wisdom_bonus / 4)
  
  local gen = target:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_color(gen:param(1.0), gen:param(0.1), gen:param(0.1), gen:param(1.0))
  effect:add_anim(gen)
  effect:apply()
end

