function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(18.0)
  targeter:set_free_select(18.0)
  targeter:set_free_select_must_be_passable(parent:size_str())
  targeter:impass_blocks_affected_points(false)
  targeter:set_shape_line_segment(parent:size_str(), parent:x(), parent:y())
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()

  local pos = targets:selected_point()
  local start_x = parent:x() + 0.5
  local start_y = parent:y() - 1.0
  
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("move_parent")
  
  local delta_x = pos.x - start_x
  local delta_y = pos.y - start_y
  local length = math.sqrt(delta_x * delta_x + delta_y * delta_y)
  local duration = 0.5
  local speed = length / duration
  local angle = game:atan2(delta_x, delta_y)

  local anim = parent:create_anim("shooting_bolt03_fast", duration)
  anim:set_completion_callback(cb)
  anim:set_draw_above_entities()
  anim:set_position(anim:param(start_x), anim:param(start_y))
  anim:set_color(anim:param(0.0), anim:param(0.5), anim:param(1.0))
  anim:set_particle_size_dist(anim:fixed_dist(length), anim:fixed_dist(2.5))
  anim:set_rotation_centroid(anim:param(start_x), anim:param(start_y + 1.25))
  anim:set_rotation(anim:param(angle))
  
  local targets = targets:to_table()
  for i = 1, #targets do 
    local dist = parent:dist_to_entity(targets[i])
    local duration = dist / speed
    
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
  
  if target:is_valid() then
    local stats = parent:stats()
	local min_dmg = 15 + stats.caster_level / 3 + stats.intellect_bonus / 6
    local max_dmg = 25 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
    parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 8, "Shock")
  end
end

function move_parent(parent, ability, targets)
  local dest = targets:selected_point()
  parent:teleport_to(dest)
  
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end
