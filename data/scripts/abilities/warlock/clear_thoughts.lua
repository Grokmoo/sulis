radius = 9.0

function on_activate(parent, ability)
  local targets = parent:targets():friendly()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_shape_circle(radius)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  
  local position = targets:selected_point()
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local target = targets[i]
	
    local anim = target:create_anim("slash", 0.35)
    anim:set_position(anim:param(target:center_x() - 1.0), anim:param(target:center_y() - 2.5))
    anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
	anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(1.0))
	anim:activate()
  
    apply_effect(parent, ability, target)
  end
end

function apply_effect(parent, ability, target)
  target:remove_effects_with_tag("charm")
  target:remove_effects_with_tag("fear")
  target:remove_effects_with_tag("dazzle")
  target:remove_effects_with_tag("sleep")

  if parent:ability_level(ability) < 2 then return end

  local stats = parent:stats()
  local amount = 20 + stats.caster_level + stats.intellect_bonus / 2
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  effect:add_num_bonus("will", amount)

  local anim = target:create_particle_generator("particles/circle12")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(-2.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.4, 0.4), anim:uniform_dist(-1.0, 1.0)),
                                  anim:dist_param(anim:uniform_dist(-0.4, 0.4), anim:uniform_dist(-1.0, 1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(0.7))
  effect:add_anim(anim)
  effect:apply()
end
