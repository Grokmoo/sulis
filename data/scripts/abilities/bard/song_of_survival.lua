SONG_NAME = "song_of_survival"

--INCLUDE common_song

function get_targets(parent)
  return parent:targets():friendly()
end

function set_anim_color(anim)
  anim:set_color(anim:param(0.35), anim:param(0.4), anim:param(0.1), anim:param(0.5))
end

function on_entered(parent, ability, targets)
  local target = targets:first()
  
  if parent:is_hostile(target) then return end
  
  local effect = target:create_effect(ability:name())
  effect:set_tag(SONG_NAME)
  
  factor = 1 + parent:get_num_flag("bard_bonus_factor")
  
  local stats = parent:stats()
  local bonus = (2 + stats.caster_level / 4 + stats.perception_bonus / 4)
  effect:add_num_bonus("armor", bonus * factor)
  
  if target:id() ~= parent:id() then
    -- don't create the hear song animation on the caster
	create_hear_anim(target, effect)
  end
  
  effect:apply()
end
