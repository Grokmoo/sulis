radius = 5.0

function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(ability:range())
  targeter:set_selection_radius(ability:range())
  targeter:set_shape_object_size("9by9round")
  targeter:invis_blocks_affected_points(true)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  game:play_sfx("sfx/fire_impact_1")
  game:play_sfx("sfx/curse2")
  
  local position = targets:selected_point()
  
  local gen = parent:create_particle_generator("wind_particle", 0.6)
  gen:set_initial_gen(1000.0)
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = radius / 0.5
  gen:set_particle_position_dist(
    gen:dist_param(
	  gen:angular_dist(0.0, 2 * math.pi, 3.0, 3.2),
	  gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)
	)
  )
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.1), gen:param(0.0), gen:param(1.0, -2.0))
  
  local targets = targets:to_table()
  
  parent:add_num_flag("__feedback_num_targets", #targets)
  
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("do_shock")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
end

function do_shock(parent, ability, targets)
  local target = targets:first()

  local duration = 0.6
  local anim = target:create_anim("shock", duration)
  anim:set_position(anim:param(target:center_x() - 1.0), anim:param(target:center_y() - 1.5))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_alpha(anim:param(1.0))
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("apply_damage")
  anim:add_callback(cb, duration - 0.2)
  anim:activate()
end

function apply_damage(parent, ability, targets)
  local target = targets:first()

  local num = parent:get_num_flag("__feedback_num_targets")
  local mult = math.sqrt(num)

  local stats = parent:stats()
  local min_dmg = (5 + stats.caster_level / 2 + stats.intellect_bonus / 4) * mult
  local max_dmg = (10 + stats.intellect_bonus / 2 + stats.caster_level) * mult
  parent:special_attack(target, "Will", "Spell", min_dmg, max_dmg, 6, "Shock")
end
