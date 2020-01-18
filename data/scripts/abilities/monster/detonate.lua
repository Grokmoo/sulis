function on_activate(parent, ability)
  local targets = parent:targets():without_self()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_selection_radius(ability:range())
  targeter:set_shape_circle(ability:range())
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  create_explosion(parent, ability, targets)
  
  local damage = parent:stats().current_hp
  
  parent:take_damage(parent, damage, damage, "Raw")
  
  ability:activate(parent)
end

function create_explosion(parent, ability, targets)
  local anim = parent:wait_anim(0.3)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_fire_surface")
  anim:set_completion_callback(cb)
  anim:activate()
  
  local duration = 1.2
  
  local position = targets:selected_point()
  
  local gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_initial_gen(500.0)
  gen:set_gen_rate(gen:param(100.0, 0, -500, -500))
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = 1.5 * ability:range() / 0.6
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * math.pi, 0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed * 1.5)
  end
  
  gen:activate()
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  
  if target:is_valid() then
    local stats = parent:stats()
	local min_dmg = 15 + stats.caster_level / 3 + stats.intellect_bonus / 6
    local max_dmg = 25 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
    parent:special_attack(target, "Reflex", "Ranged", min_dmg, max_dmg, 0, "Fire")
  end
end

function create_fire_surface(parent, ability, targets)
  local points = targets:random_affected_points(0.7)
  fire_surface(parent, ability, points, 2)
end

--INCLUDE fire_surface