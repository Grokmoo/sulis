function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_selectable(parent)
  targeter:set_shape_circle(ability:range())
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  local position = targets:selected_point()
  local gen = parent:create_particle_generator("wind_particle", 0.6)
  gen:set_initial_gen(1000.0)
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = ability:range() / 0.5
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.5), gen:param(0.5), gen:param(1.0, -2.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("apply_effect")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
  
  game:play_sfx("sfx/wind_effects_5")
end

function apply_effect(parent, ability, targets)
  local target = targets:first()
  local stats = parent:stats()
  local amount = 5 + stats.caster_level / 2 + stats.intellect_bonus / 4
  
  local hit = parent:special_attack(target, "Fortitude", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount * 0.5
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("hex")
  effect:add_num_bonus("movement_rate", -(amount / 10))
  effect:add_num_bonus("defense", -amount)

  local gen = target:create_anim("slow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0))
  effect:add_anim(gen)
  effect:apply()
end
