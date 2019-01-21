radius = 8.0

function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(targets)
  targeter:allow_affected_points_impass(false)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  local duration = 1.2
  
  create_acid_surface(parent, ability, targets)
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local gen = parent:create_particle_generator("particles/circle4", duration)
    gen:set_initial_gen(200.0)
    gen:set_gen_rate(gen:param(20.0, 0, -500, -500))
    gen:set_position(gen:param(targets[i]:center_x()), gen:param(targets[i]:center_y()))
    gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
    local speed = 2.0 / 0.6
    gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * math.pi, 0, speed)))
    gen:set_particle_duration_dist(gen:fixed_dist(0.6))
    gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(0.2), gen:param(1.0, -1.8))
  
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, 0.5)
	
    gen:activate()
  end
end

function attack_target(parent, ability, targets)
  local target = targets:first()

  if not target:is_valid() then return end
  
  local stats = parent:stats()
  local min_dmg = 5 + stats.intellect_bonus / 8 + stats.caster_level / 4
  local max_dmg = 10 + stats.intellect_bonus / 4 + stats.caster_level / 2
  
  local hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 8, "Acid")
  local duration = 3
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = 4
  end
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("damage")
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_damage")
  effect:add_callback(cb)
  
  local anim = target:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_initial_gen(10.0)
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.2))
  anim:set_gen_rate(anim:param(20.0))
  anim:set_position(anim:param(0.0), anim:param(-0.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
  anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
end

function apply_damage(parent, ability, targets)
  local target = targets:first()
  
  local stats = parent:stats()
  local min_dmg = 5 + stats.caster_level / 4 + stats.intellect_bonus / 8
  local max_dmg = 10 + stats.intellect_bonus / 4 + stats.caster_level / 2
  target:take_damage(parent, min_dmg, max_dmg, "Acid", 8)
end

function create_acid_surface(parent, ability, targets)
  local points = targets:random_affected_points(0.6)
  local surf = parent:create_surface(ability:name(), points, 4)
  surf:set_squares_to_fire_on_moved(3)
  
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surf:add_callback(cb)
  
  local gen = parent:create_particle_generator("particles/circle8")
  gen:set_alpha(gen:param(0.75))
  gen:set_gen_rate(gen:param(30.0))
  gen:set_position(gen:param(0.0), gen:param(0.0))
  gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(0.3))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.5, 0.5)),
								 gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.5, 0.5)))
  gen:set_draw_below_entities()
  surf:add_anim(gen)
  
  surf:apply()
end

function on_moved(parent, ability, targets)
  local target = targets:first()
  target:take_damage(parent, 2, 4, "Acid", 3)
end

function on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:take_damage(parent, 2, 4, "Acid", 3)
  end
end
