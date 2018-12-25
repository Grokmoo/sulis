function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  speed = 30.0
  dist = parent:dist_to_entity(target)
  duration = 0.5 + dist / speed
  parent_center_y = parent:center_y() - 1.0
  vx = (target:center_x() - parent:center_x()) / duration
  vy = (target:center_y() - parent_center_y) / duration
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  
  gen = parent:create_particle_generator("particles/circle8", duration)
  gen:set_position(gen:param(parent:center_x(), vx), gen:param(parent_center_y, vy))
  gen:set_gen_rate(gen:param(70.0))
  gen:set_initial_gen(35.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vx / 5.0, 0.0)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vy / 5.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(0.1))
  gen:add_callback(cb, duration - 0.1)
  gen:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()
  
  hit = parent:special_attack(target, "Reflex", "Ranged", 10, 20, 8, "Acid")
  amount = -8
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  effect = target:create_effect(ability:name(), 2)
  effect:set_tag("sundered_armor")
  effect:add_num_bonus("armor", amount)
  
  anim = target:create_color_anim()
  anim:set_color(anim:param(0.4),
                 anim:param(1.0),
                 anim:param(0.4),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.0),
                     anim:param(0.5),
                     anim:param(0.1),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()
end
