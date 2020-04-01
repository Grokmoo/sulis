function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_selection_radius(radius(parent, ability))
  targeter:set_shape_circle(radius(parent, ability))
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function radius(parent, ability)
  local radius = ability:range()
  if parent:ability_level(ability) > 1 then
    radius = radius + 2.0
  end
  return radius
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  
  local position = targets:selected_point()
  
  local gen = parent:create_particle_generator("wind_particle", 0.6)
  gen:set_initial_gen(1000.0)
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = radius(parent, ability) / 0.5
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.1), gen:param(0.0), gen:param(1.0, -2.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("apply_effect")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
  game:play_sfx("sfx/magicfail2", 2.0)
end

function apply_effect(parent, ability, targets)
  local target = targets:first()
  
  local stats = parent:stats()
  local hit = parent:special_attack(target, "Will", "Spell")
  local amount = -(5 + stats.intellect_bonus / 10) * game:ap_display_factor()
  
  if parent:ability_level(ability) > 1 then
    amount = amount + game:ap_display_factor()
  end
  
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
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
  game:play_sfx("sfx/thwack-01")
end
