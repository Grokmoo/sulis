SONG_NAME = "song_of_survival"

--INCLUDE common_song

function get_targets(parent)
  return parent:targets():friendly()
end

function set_anim_color(anim)
  anim:set_color(anim:param(0.35), anim:param(0.4), anim:param(0.1), anim:param(0.5))
  
  game:play_sfx("sfx/guitar")
  game:play_sfx("sfx/song_good")
end

function on_entered(parent, ability, targets)
  local target = targets:first()
  
  local effect = target:create_effect(ability:name())
  effect:set_tag(SONG_NAME)
  
  if not parent:is_hostile(target) then
    factor = 1 + parent:get_num_flag("bard_bonus_factor")
  
    local stats = parent:stats()
    local bonus = (2 + stats.caster_level / 4 + stats.perception_bonus / 4)
    effect:add_num_bonus("armor", bonus * factor)
  end
  
  create_hear_anim(parent, target, effect)

  effect:apply()
end
