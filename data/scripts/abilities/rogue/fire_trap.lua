function on_activate(parent, ability)
  if game:num_effects_with_tag("trap") > 4 then
    game:say_line("Maximum number of traps set.", parent)
    return
  end

  local targets = parent:targets()
  local radius = parent:stats().touch_distance
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(radius)
  targeter:set_free_select(radius)
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("1by1")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()

  local surf = parent:create_surface(ability:name(), points)
  surf:set_tag("trap")
  surf:set_squares_to_fire_on_moved(1)
  
  local cb = ability:create_callback(parent)
  cb:set_on_moved_in_surface_fn("on_entered")
  surf:add_callback(cb)
  
  local anim = parent:create_anim("particles/fire_trap")
  anim:set_position(anim:param(0.0), anim:param(0.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_draw_below_entities()
  surf:add_anim(anim)
  
  surf:apply()
  ability:activate(parent)
  
  game:play_sfx("sfx/click_1")
end

function on_entered(parent, ability, targets)
  -- only fire on hostiles
  if targets:hostile():is_empty() then return end
  
  game:play_sfx("sfx/fire_impact_1")
  
  targets:surface():mark_for_removal()
  
  local target = targets:first()
  local points = targets:affected_points()
  local point = points[1]
  
  local gen = target:create_particle_generator("fire_particle", 1.0)
  gen:set_initial_gen(200.0)
  gen:set_gen_rate(gen:param(50.0, 0, -200, -200))
  gen:set_position(gen:param(point.x), gen:param(point.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-0.5, 0.5)),
    gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-2.0, -10.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:activate()
  
  local stats = parent:stats()
  local min_dmg = 25 + stats.level / 2 + stats.intellect_bonus / 4
  local max_dmg = 35 + stats.level + stats.intellect_bonus / 2
  
  if parent:has_ability("mechanical_mastery") then
    min_dmg = min_dmg + 5
	max_dmg = max_dmg + 7
  end
  
  parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 0, "Fire")
  
  if parent:ability_level(ability) > 1 then
    local nearby = target:targets():visible_within(3):to_table()
	for i = 1, #nearby do
      parent:special_attack(nearby[i], "Reflex", "Ranged", 10, 20 + stats.level / 2, 0, "Fire")
    end
	
	local gen = target:create_particle_generator("fire_particle", 1.0)
    gen:set_initial_gen(50.0)
    gen:set_gen_rate(gen:param(10.0, 0, -100, -100))
    gen:set_position(gen:param(point.x), gen:param(point.y))
    gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
    gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-5.0, 5.0)),
    gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-2.0, -10.0)))
    gen:set_particle_duration_dist(gen:fixed_dist(0.6))
    gen:activate()
  end
end