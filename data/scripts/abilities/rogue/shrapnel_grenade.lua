frag_radius = 4.0

function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("7by7round")
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local selected_point = targets:selected_point()
  local speed = 20.0
  local dist = parent:dist_to_point(selected_point)
  local duration = dist / speed
  local vx = (selected_point.x - parent:center_x()) / duration
  local vy = (selected_point.y - parent:center_y()) / duration
  
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_explosion")
  
  local gen = parent:create_anim("particles/circle12", duration)
  gen:set_color(gen:param(0.5), gen:param(0.5), gen:param(0.5))
  gen:set_position(gen:param(parent:center_x(), vx), gen:param(parent:center_y(), vy))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vx / 5.0)),
    gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vy / 5.0)))
  gen:set_completion_callback(cb)
  gen:activate()
  
  ability:activate(parent)
  game:play_sfx("sfx/swish-7")
end

function create_explosion(parent, ability, targets)
  game:play_sfx("sfx/explode5")

  local duration = 1.2
  
  local position = targets:selected_point()
  
  local gen = parent:create_particle_generator("particles/circle4", duration)
  gen:set_initial_gen(100.0)
  gen:set_gen_rate(gen:param(20.0, 0, -500, -500))
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
  local speed = 1.5 * frag_radius / 0.6
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * math.pi, 0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_color(gen:param(0.5), gen:param(0.5), gen:param(0.5))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  if target:is_valid() then
    local stats = parent:stats()
	local min_dmg = 12 + stats.level / 4 + stats.intellect_bonus / 6
    local max_dmg = 24 + stats.intellect_bonus / 3 + stats.level / 2
	
	if parent:has_ability("mechanical_mastery") then
      min_dmg = min_dmg + 4
	  max_dmg = max_dmg + 6
    end
	
    parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 8, "Piercing")
  end
end
