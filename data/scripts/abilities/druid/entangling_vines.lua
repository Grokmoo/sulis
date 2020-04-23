function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:set_shape_circle(8.0)
  targeter:allow_affected_points_impass(false)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points, ability:duration())
  
  local stats = parent:stats()
  local bonus = (10 + stats.caster_level / 2 + stats.wisdom_bonus / 4) * 0.03
  surface:add_num_bonus("movement_rate", -bonus)
  surface:add_num_bonus("move_anim_rate", -0.3)

  surface:set_squares_to_fire_on_moved(6)
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surface:add_callback(cb)
  
  local gen = parent:create_anim("vines")
  gen:set_position(gen:param(0.0), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.5))
  gen:set_particle_frame_time_offset_dist(gen:uniform_dist(0.0, 0.9))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.4, 0.4)),
                                 gen:dist_param(gen:uniform_dist(-0.4, 0.4)))
  gen:set_draw_below_entities()
  surface:add_anim(gen)
  surface:apply()
  
  ability:activate(parent)
  game:play_sfx("sfx/rustle07")
end

function on_moved(parent, ability, targets)
  try_hold(parent, ability, targets:first())
end

function on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    targets[i]:remove_effects_with_tag("tangle")
	try_hold(parent, ability, targets[i])
  end
end

function try_hold(parent, ability, target)
  if target:has_effect_with_tag("tangle") then return end

  local hit = parent:special_attack(target, "Reflex", "Spell")
  
  if hit:is_miss() then return end
  
  game:play_sfx("sfx/rustle09")
  
  local effect = target:create_effect(ability:name(), 1)
  effect:set_tag("tangle")
  
  if hit:is_graze() then
    effect:add_num_bonus("defense", -5)
	effect:add_num_bonus("reflex", -5)
  elseif hit:is_hit() then
    effect:add_move_disabled()
  elseif hit:is_crit() then
    effect:add_move_disabled()
	effect:add_num_bonus("defense", -10)
	effect:add_num_bonus("reflex", -10)
  end
  
  local gen = target:create_anim("slow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()
end
