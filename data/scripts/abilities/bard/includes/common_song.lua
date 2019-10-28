-- This file is included by the bard songs.

function on_activate(parent, ability)
  if parent:has_active_mode() then
    game:say_line("Only one song may be active at a time.", parent)
    return
  end

  local targets = get_targets(parent)
  
  local targeter = parent:create_targeter(ability)
  targeter:add_selectable(parent)
  targeter:set_shape_circle(8.0 + parent:ability_level_from_id("louder_music") * 2)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)

  -- deactivate any currently active melodies along with the main song
  melodies = { "funeral_dirge", "disharmonious_melody" }
  for i = 1, #melodies do
	local melody_ability = parent:get_ability(melodies[i])
	if melody_ability ~= nil then
	  if melody_ability:is_active_mode(parent) then
	    local anim = parent:wait_anim(0.0)
	    local cb = melody_ability:create_callback(parent)
        cb:set_on_anim_complete_fn("on_deactivate")
        anim:set_completion_callback(cb)
        anim:activate()
	  end
	end
  end
end

function reactivate(parent, ability, targets)
  create_song(parent, ability, targets)
  ability:activate(parent, false) -- don't use AP
end

function on_target_select(parent, ability, targets)
  create_song(parent, ability, targets)
  ability:activate(parent)
end

function create_song(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points)
  surface:set_tag(SONG_NAME .. "_surface")
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
  effect:set_tag("singing_" .. SONG_NAME)
  effect:deactivate_with(ability)
  effect:set_ui_visible(true)

  check_rhythms(parent, ability, effect)

  local anim = parent:create_particle_generator("note")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.7), anim:fixed_dist(0.7))
  anim:set_gen_rate(anim:param(4.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  set_anim_color(anim)
  effect:add_anim(anim)
  effect:apply()
end

function check_rhythms(parent, ability, effect)
  local flag = parent:get_flag("bard_rhythm")
  
  local ap_bonus = -1 * game:ap_display_factor()
  
  if flag == nil then
    parent:clear_flag("bard_bonus_factor")
	effect:add_num_bonus("ap", ap_bonus)
	return
  end
  
  if flag == "driving" then
    effect:add_num_bonus("ap", 2 * ap_bonus)
	parent:add_num_flag("bard_bonus_factor", 0.5)
  elseif flag == "downbeat" then
	parent:add_num_flag("bard_bonus_factor", -0.25)
  elseif flag == "progressive" then
    effect:add_num_bonus("ap", ap_bonus)
    parent:add_num_flag("bard_bonus_factor", 0.5)
	effect:add_num_bonus("movement_rate", -0.4)
	effect:add_num_bonus("defense", -25)
	effect:add_num_bonus("melee_accuracy", -25)
	effect:add_num_bonus("ranged_accuracy", -25)
  elseif flag == "syncopated" then
    effect:add_num_bonus("ap", ap_bonus)
    parent:add_num_flag("bard_bonus_factor", 0.5)
  end

  local all_rhythm = parent:get_abilities_with_group("Rhythm")
  for i = 1, #all_rhythm do
    all_rhythm[i]:cooldown(parent, 3)
  end
  
  parent:clear_flag("bard_rhythm")
end

function on_moved(parent, ability, targets)
   local target = targets:first()
end

function on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    
  end
end

function create_hear_anim(parent, target, effect)
  -- don't create hear animation on caster
  if parent:id() == target:id() then return end

  local anim = target:create_particle_generator("hear")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.4), anim:fixed_dist(0.4))
  anim:set_gen_rate(anim:param(4.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  
  if parent:is_hostile(target) then
    anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0), anim:param(0.5))
  else
    anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(0.5))
  end
  
  effect:add_anim(anim)
end

function on_exited(parent, ability, targets)
  local target = targets:first()
  target:remove_effects_with_tag(SONG_NAME)
end