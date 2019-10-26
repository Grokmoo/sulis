SONG_NAME="song_of_heroes"

--INCLUDE common_song

function get_targets(parent)
  return parent:targets():friendly()
end

function set_anim_color(anim)
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0), anim:param(0.5))
end

function on_entered(parent, ability, targets)
  local target = targets:first()
  
  local effect = target:create_effect(ability:name())
  effect:set_tag(SONG_NAME)
  
  if not parent:is_hostile(target) then
    factor = 1 + parent:get_num_flag("bard_bonus_factor")
    
    local stats = parent:stats()
    local bonus = (10 + stats.caster_level / 2 + stats.perception_bonus / 2) * factor
    effect:add_num_bonus("melee_accuracy", bonus)
    effect:add_num_bonus("ranged_accuracy", bonus)
    effect:add_num_bonus("spell_accuracy", bonus)
  end
  
  create_hear_anim(parent, target, effect)
  
  effect:apply()
end