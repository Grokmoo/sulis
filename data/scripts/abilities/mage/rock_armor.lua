function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  
  local stats = parent:stats()
  effect:add_num_bonus("armor", 12 + stats.caster_level)
  effect:add_attribute_bonus("Dexterity", -4)

  local anim = parent:create_color_anim()
  anim:set_color(anim:param(0.5),
                 anim:param(0.6),
                 anim:param(0.5),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.3),
                     anim:param(0.2),
                     anim:param(0.2),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()

  ability:activate(parent)
  game:play_sfx("sfx/echo03")
end
