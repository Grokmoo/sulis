-- This file is included in the bard rhythms

function on_activate(parent, ability)
  local songs = { "song_of_heroes", "song_of_curses", "song_of_survival" }
  
  local song_ability = nil
  local points = nil
  for i = 1, #songs do
    local song = songs[i]
    if parent:has_effect_with_tag("singing_" .. song) then
	  song_ability = parent:get_ability(song)
	  local song_effects = parent:get_auras_with_tag(song .. "_surface")
	  points = song_effects[1]:surface_points()
	end
  end

  if song_ability == nil then
    game:say_line("You must have a bard song active.", parent)
    return
  end
  
  song_ability:deactivate(parent) -- call deactivate directly, bypassing the on_deactivate script on the ability
  parent:set_flag("bard_rhythm", RHYTHM_TYPE)
  
  local cb = song_ability:create_callback(parent)
  cb:add_affected_points(points)
  cb:set_on_anim_complete_fn("reactivate")
  local anim = parent:wait_anim(0.25)
  anim:set_completion_callback(cb)
  anim:activate()

  local gen = parent:create_particle_generator("note", 1.0)
  gen:set_initial_gen(20.0)
  gen:set_position(gen:param(parent:center_x()), gen:param(parent:center_y() - 0.5))
  gen:set_gen_rate(gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.4, 0.4), gen:uniform_dist(-2.0, 2.0)),
                                  gen:dist_param(gen:uniform_dist(-0.4, 0.4), gen:uniform_dist(-2.0, 2.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(1.0))
  gen:set_alpha(gen:param(1.0, -1.0))
  gen:activate()

  ability:activate(parent)
end
