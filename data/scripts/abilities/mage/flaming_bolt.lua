function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_visible()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local speed = 16.0
  local dist = parent:dist_to_entity(target)
  local duration = dist / speed
  local parent_center_y = parent:center_y() - 1.0
  local vx = (target:center_x() - parent:center_x()) / duration
  local vy = (target:center_y() - parent_center_y) / duration
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  
  local gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:center_x(), vx), gen:param(parent_center_y, vy))
  gen:set_gen_rate(gen:param(50.0))
  gen:set_initial_gen(35.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:uniform_dist(-vx / 8.0, 0.0)),
    gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:uniform_dist(-vy / 8.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:add_callback(cb, duration - 0.1)
  gen:activate()
  
  ability:activate(parent)
  game:play_sfx("sfx/spell")
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  
  parent:special_attack(target, "Reflex", "Ranged", 25, 35, 0, "Fire")
  
  local gen = target:create_particle_generator("fire_particle", 0.6)
  gen:set_initial_gen(50.0)
  gen:set_position(gen:param(target:center_x()), gen:param(target:center_y()))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0), gen:fixed_dist(5.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:activate()
  
  game:play_sfx("sfx/explode5")
end
