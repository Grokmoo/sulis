line_len = 20.0

function on_activate(parent, ability)
  local stats = parent:stats()
  if not stats.attack_is_ranged then
    game:say_line("You must have a ranged weapon equipped.", parent)
    return
  end

  local targets = parent:targets():without_self()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(line_len)
  targeter:set_free_select(line_len)
  targeter:set_shape_line("1by1", parent:x(), parent:y(), line_len)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  
  local speed = 500 * game:anim_base_time()
  local duration = line_len / speed
  
  local anim = parent:create_anim(parent:stats().ranged_projectile, duration)

  local delta_x = pos.x - parent:x()
  local delta_y = pos.y - parent:y()
  local angle = game:atan2(delta_x, delta_y)
  
  local norm = math.sqrt((delta_x * delta_x) + (delta_y * delta_y))
  
  delta_x = delta_x / norm * line_len
  delta_y = delta_y / norm * line_len
  
  anim:set_position(anim:param(parent:x(), delta_x / duration), anim:param(parent:y(), delta_y / duration))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_rotation(anim:param(angle))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do 
    local dist = parent:dist_to_entity(targets[i])
    duration = dist / speed
    
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    anim:add_callback(cb, duration)
  end
  
  anim:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  local stats = parent:stats()
  
  if target:is_valid() then
    parent:special_attack(target, "Reflex", "Ranged", stats.damage_min_0, stats.damage_max_0, stats.armor_penetration_0, "Raw")
  end
end

