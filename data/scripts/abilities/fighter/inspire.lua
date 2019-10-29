radius = 6.0

function on_activate(parent, ability)
  local targets = parent:targets():friendly()
  effectable = targets:without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(radius)
  targeter:add_selectable(parent)
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(effectable)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  
  local position = targets:selected_point()
  
  local gen = parent:create_particle_generator("wind_particle", 0.6)
  gen:set_initial_gen(1000.0)
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = radius / 0.5
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.1), gen:param(0.1), gen:param(1.0, -2.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("apply_effect")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
end

function apply_effect(parent, ability, targets)
  local target = targets:first()
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:add_attribute_bonus("Strength", 2)
  effect:add_attribute_bonus("Dexterity", 2)
  effect:add_attribute_bonus("Endurance", 2)
  effect:add_attribute_bonus("Perception", 2)
  effect:add_attribute_bonus("Intellect", 2)
  effect:add_attribute_bonus("Wisdom", 2)

  local anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(-2.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.1), anim:param(0.5))
  effect:add_anim(anim)
  effect:apply()
end
