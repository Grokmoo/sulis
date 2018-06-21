max_dist = 10

function on_activate(parent, ability)
  targets = parent:targets():without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(max_dist * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), max_dist, math.pi / 3) 
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  pos = targets:selected_point()
  
  delta_x = pos.x - parent:center_x()
  delta_y = pos.y - parent:center_y()
  angle = game:atan2(delta_x, delta_y)
  
  duration = 1.5
  
  gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:center_x()), gen:param(parent:center_y()))
  gen:set_gen_rate(gen:param(500.0, -500))
  gen:set_initial_gen(500.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(
    gen:dist_param(gen:uniform_dist(-0.1, 0.1),
    gen:angular_dist(angle - math.pi / 6, angle + math.pi / 6, 0, 20)))
    
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  targets = targets:to_table()
  for i = 1, #targets do
    dist = parent:dist_to_entity(targets[i])
    cb_dur = duration * dist / max_dist
    
    cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, cb_dur)
  end
  
  gen:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()

  if target:is_valid() then
    parent:special_attack(target, "Reflex", 20, 30, 0, "Fire")
  end
end
