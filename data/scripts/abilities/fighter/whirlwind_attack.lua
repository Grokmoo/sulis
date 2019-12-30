function on_activate(parent, ability)
  local targets = parent:targets():hostile():attackable()

  local targeter = parent:create_targeter(ability)
  targeter:set_show_mouseover(false)
  targeter:set_selection_attackable()
  targeter:set_shape_circle(parent:stats().attack_distance)
  targeter:add_selectable(parent)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local duration = 1.8
  local gen = parent:create_anim("whirlwind", duration)
  gen:set_position(gen:param(parent:center_x() - 2.5), gen:param(parent:center_y() - 3.0))
  gen:set_particle_size_dist(gen:fixed_dist(6.0), gen:fixed_dist(6.0))
  gen:set_alpha(gen:param(1.0, 0.0, 0.0, -6.0))

  local targets = targets:to_table()
  local duration_per_target = 0.5 * duration / (#targets + 1)
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, duration_per_target * i)
  end
  
  gen:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  if target:is_valid() then
    parent:weapon_attack(target)
  end
end

