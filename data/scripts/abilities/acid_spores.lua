radius = 8.0

function on_activate(parent, ability)
  targets = parent:targets():hostile()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  duration = 1.2
  
  targets = targets:to_table()
  for i = 1, #targets do
    gen = parent:create_particle_generator("particles/circle4", duration)
    gen:set_initial_gen(200.0)
    gen:set_gen_rate(gen:param(20.0, 0, -500, -500))
    gen:set_position(gen:param(targets[i]:center_x()), gen:param(targets[i]:center_y()))
    gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
    speed = 2.0 / 0.6
    gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * math.pi, 0, speed)))
    gen:set_particle_duration_dist(gen:fixed_dist(0.6))
    gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(0.2), gen:param(1.0, -1.8))
  
    cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, 0.5)
	
    gen:activate()
  end
end

function attack_target(parent, ability, targets)
  target = targets:first()

  if not target:is_valid() then return end
  
  hit = parent:special_attack(target, "Reflex", 10, 15, 0, "Acid")
  duration = 3
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = 4
  end
  
  effect = target:create_effect(ability:name(), duration)
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_damage")
  effect:add_callback(cb)
  
  anim = target:create_particle_generator("particles/circle4")
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
  target = targets:first()
  
  target:take_damage(10, 15, "Acid")
end
