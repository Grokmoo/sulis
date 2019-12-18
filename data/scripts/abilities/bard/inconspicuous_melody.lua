MELODY_NAME = "inconspicuous_melody"

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
  surface:set_tag(MELODY_NAME)
  surface:deactivate_with(ability)
  surface:set_ui_visible(false)
  surface:set_aura(parent)
  local cb = ability:create_callback(parent)
  cb:set_on_entered_surface_fn("on_entered")
  cb:set_on_exited_surface_fn("on_exited")
  surface:add_callback(cb)
  surface:apply()
  
  local anim = parent:create_anim("note_electric", 0.7)
  anim:set_position(anim:param(parent:center_x() - 1.0), anim:param(parent:center_y() - 2.0))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_color(anim:param(0.0), anim:param(0.5), anim:param(0.0))
  anim:activate()
  
  local effect = parent:create_effect(ability:name())
  effect:set_tag(MELODY_NAME)
  effect:deactivate_with(ability)
  effect:add_num_bonus("ap", -1 * game:ap_display_factor())
  effect:apply()
  
  ability:activate(parent)
end

function on_deactivate(parent, ability)
  -- guard against double call since this is called from the common_song script
  if ability:is_active_mode(parent) then
    ability:deactivate(parent)
  end
end

function on_entered(parent, ability, targets)
  local target = targets:first()
  
  if parent:is_hostile(target) then return end
  
  local effect = target:create_effect(ability:name())
  effect:set_tag(MELODY_NAME)
  
  local stats = parent:stats()
  local bonus = (20 + stats.caster_level / 2 + stats.perception_bonus / 2)
  effect:add_num_bonus("concealment", bonus)
  
  effect:apply()
end

function on_exited(parent, ability, targets)
  local target = targets:first()
  target:remove_effects_with_tag(MELODY_NAME)
end