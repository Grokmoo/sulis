function on_activate(parent, ability)
  local targets = parent:targets():without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range() * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), 1.0, ability:range(), math.pi / 4) 
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  
  local delta_x = pos.x - parent:center_x()
  local delta_y = pos.y - parent:center_y()
  local angle = game:atan2(delta_x, delta_y)
  
  local duration = 1.5
  
  gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_position(gen:param(parent:center_x()), gen:param(parent:center_y()))
  gen:set_gen_rate(gen:param(0.0))
  gen:set_initial_gen(500.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(
    gen:dist_param(gen:uniform_dist(-0.1, 0.1),
    gen:angular_dist(angle - math.pi / 8, angle + math.pi / 8, 22, 30)))
    
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local dist = parent:dist_to_entity(targets[i])
    local cb_dur = 0.5 * duration * (1 - dist / ability:range())
    -- fire callback for further targets first, so they move out of the way of the closer targets
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, cb_dur)
  end
  
  gen:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  if not target:is_valid() then return end
  
  local stats = parent:stats()
  local hit = parent:special_attack(target, "Reflex", "Spell")
  
  local base_dist = math.floor(8 + stats.caster_level / 3, stats.intellect_bonus / 6 - target:width())
  local point = { x = parent:x(), y = parent:y() }
  local direction = 1
  
  local pushback = push_target(base_dist, target, hit, point, direction)
  
  if pushback > 0 then
    target:take_damage(parent, pushback * 2 - 2, pushback * 2 + 2, "Crushing")
  end
end

--INCLUDE push_target
