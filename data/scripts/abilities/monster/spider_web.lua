radius = 4.0

function on_activate(parent, ability)
  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(6.0)
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  points = targets:affected_points()
  surface = parent:create_surface(ability:name(), points, ability:duration())
  surface:set_squares_to_fire_on_moved(6)
  
  cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surface:add_callback(cb)
  
  s_anim = parent:create_particle_generator("particles/web16")
  s_anim:set_position(s_anim:param(0.0), s_anim:param(0.0))
  s_anim:set_initial_gen(1.0)
  s_anim:set_gen_rate(s_anim:param(2.0))
  s_anim:set_particle_size_dist(s_anim:fixed_dist(1.0), s_anim:fixed_dist(1.0))
  s_anim:set_particle_duration_dist(s_anim:uniform_dist(0.3, 0.7))
  s_anim:set_particle_position_dist(s_anim:dist_param(s_anim:uniform_dist(-0.3, 0.3)),
                                    s_anim:dist_param(s_anim:uniform_dist(-0.3, 0.3)))
  s_anim:set_draw_below_entities()
  surface:add_anim(s_anim)
  surface:apply()
  
  targets = targets:to_table()
  for i = 1, #targets do
	web_target(parent, ability, targets[i])
  end
end

function on_moved(parent, ability, targets)
  web_target(parent, ability, targets:first())
end

function on_round_elapsed(parent, ability, targets)
  targets = targets:to_table()
  for i = 1, #targets do
	web_target(parent, ability, targets[i])
  end
end

function web_target(parent, ability, target)
  if target:has_ability("spider_immunity") then return end

  hit = parent:special_attack(target, "Reflex", "Ranged")
  if hit:is_miss() then return end

  effect = target:create_effect(ability:name(), 1)
  effect:set_tag("slow")
  
  if hit:is_graze() then
    effect:add_num_bonus("movement_rate", -0.5)
	effect:add_num_bonus("defense", -5)
	effect:add_num_bonus("reflex", -5)
  elseif hit:is_hit() then
    effect:add_num_bonus("movement_rate", -0.75)
	effect:add_num_bonus("defense", -10)
	effect:add_num_bonus("reflex", -10)
  elseif hit:is_crit() then
    effect:add_move_disabled()
	effect:add_num_bonus("defense", -20)
	effect:add_num_bonus("reflex", -20)
  end
  
  gen = target:create_anim("slow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()
end