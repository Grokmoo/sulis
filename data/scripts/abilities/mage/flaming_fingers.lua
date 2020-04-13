function on_activate(parent, ability)
  local targets = parent:targets():without_self()
  
  local min_dist = math.max(parent:width(), parent:height()) / 2.0
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range() * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), min_dist, ability:range(), math.pi / 3) 
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:wait_anim(0.3)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_fire_surface")
  anim:set_completion_callback(cb)
  anim:activate()

  local pos = targets:selected_point()
  
  local delta_x = pos.x - parent:x()
  local delta_y = pos.y - parent:y()
  local angle = game:atan2(delta_x, delta_y)
  
  local duration = 1.5
  
  local gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:center_x() + 0.5), gen:param(parent:center_y() - 0.5))
  gen:set_gen_rate(gen:param(500.0, -500))
  gen:set_initial_gen(500.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(
    gen:dist_param(gen:uniform_dist(-0.1, 0.1),
    gen:angular_dist(angle - math.pi / 6, angle + math.pi / 6, 0, 20)))
    
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  local targets_table = targets:to_table()
  for i = 1, #targets_table do
    local dist = parent:dist_to_entity(targets_table[i])
    local cb_dur = duration * dist / ability:range()
    
    local cb = ability:create_callback(parent)
	cb:add_target(targets_table[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, cb_dur)
  end
  
  gen:activate()
  ability:activate(parent)
  
  game:play_sfx("sfx/flamethrower")
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  if target:is_valid() then
    local stats = parent:stats()
	local min_dmg = 8 + stats.caster_level / 2 + stats.intellect_bonus / 4
	local max_dmg = 16 + stats.caster_level + stats.intellect_bonus / 2
  
    parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 0, "Fire")
  end
end

function create_fire_surface(parent, ability, targets)
  local points = targets:random_affected_points(0.3)
  fire_surface(parent, ability, points, 1)
end

--INCLUDE fire_surface