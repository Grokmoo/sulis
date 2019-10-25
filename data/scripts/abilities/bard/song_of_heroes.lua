function on_activate(parent, ability)
  if parent:has_active_mode() then
    game:say_line("Only one song may be active at a time.", parent)
    return
  end

  local targets = parent:targets():friendly()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_shape_circle(8.0)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points)
  surface:deactivate_with(ability)
  surface:set_squares_to_fire_on_moved(6)
  surface:set_ui_visible(false)
  surface:set_aura(parent)
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  cb:set_on_entered_surface_fn("on_entered")
  cb:set_on_exited_surface_fn("on_exited")
  surface:add_callback(cb)
  surface:apply()

  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:set_ui_visible(false)

  local anim = parent:create_particle_generator("note")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.7), anim:fixed_dist(0.7))
  anim:set_gen_rate(anim:param(4.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(0.5))
  effect:add_anim(anim)
  effect:apply()

  ability:activate(parent)
end

function on_moved(parent, ability, targets)
   local target = targets:first()
end

function on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    
  end
end

function on_entered(parent, ability, targets)
  local target = targets:first()
  
  if parent:is_hostile(target) then return end
  
  local effect = target:create_effect(ability:name())
  effect:set_tag("song_of_heroes")
  
  local stats = parent:stats()
  local bonus = (10 + stats.caster_level / 2 + stats.perception_bonus / 4)
  effect:add_num_bonus("defense", bonus)
  effect:add_num_bonus("fortitude", bonus)
  effect:add_num_bonus("reflex", bonus)
  effect:add_num_bonus("will", bonus)
  
  if target:id() ~= parent:id() then
    -- don't create the hear song animation on the caster
	local anim = target:create_particle_generator("hear")
    anim:set_moves_with_parent()
    anim:set_position(anim:param(-0.5), anim:param(-1.5))
    anim:set_particle_size_dist(anim:fixed_dist(0.4), anim:fixed_dist(0.4))
    anim:set_gen_rate(anim:param(4.0))
    anim:set_initial_gen(2.0)
    anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                    anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
    anim:set_particle_duration_dist(anim:fixed_dist(1.0))
    anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(0.5))
    effect:add_anim(anim)
  end
  
  effect:apply()
end

function on_exited(parent, ability, targets)
  local target = targets:first()
  target:remove_effects_with_tag("song_of_heroes")
end