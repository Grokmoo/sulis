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
  
  local surface = parent:create_surface(ability:name(), points)
  surface:set_tag("funeral_dirge")
  surface:deactivate_with(ability)
  surface:set_ui_visible(false)
  surface:set_aura(parent)
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  surface:add_callback(cb)
  surface:apply()
  
  local anim = parent:create_anim("note_electric", 0.7)
  anim:set_position(anim:param(parent:center_x() - 1.0), anim:param(parent:center_y() - 2.0))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_color(anim:param(0.3), anim:param(0.3), anim:param(0.3))
  anim:activate()
  
  local effect = parent:create_effect(ability:name())
  effect:set_tag("funeral_dirge")
  effect:deactivate_with(ability)
  effect:add_num_bonus("ap", -1 * game:ap_display_factor())
  effect:apply()
  
  game:play_sfx("sfx/SAX11")
  game:play_sfx("sfx/song_bad")
  
  ability:activate(parent)
end

function on_deactivate(parent, ability)
  -- guard against double call since this is called from the common_song script
  if ability:is_active_mode(parent) then
    ability:deactivate(parent)
  end
end

function on_round_elapsed(parent, ability, targets)
  local stats = parent:stats()
  local min_damage = (1 + stats.caster_level / 4 + stats.perception_bonus / 2)

  local targets = targets:to_table()
  for i = 1, #targets do
    local target = targets[i]
	if parent:is_hostile(target) then
	  target:take_damage(parent, min_damage, min_damage * 2, "Piercing", 4)
	end
  end
end