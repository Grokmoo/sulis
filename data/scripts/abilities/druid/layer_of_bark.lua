function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  
  stats = parent:stats()
  bonus = stats.caster_level + stats.wisdom_bonus
  effect:add_resistance(25 + bonus, "Slashing")
  effect:add_resistance(30 + bonus, "Piercing")
  effect:add_resistance(20 + bonus, "Crushing")
  effect:add_resistance(-25, "Fire")

  anim = parent:create_color_anim()
  anim:set_color(anim:param(0.54),
                 anim:param(0.27),
                 anim:param(0.07),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.54),
                     anim:param(0.27),
                     anim:param(0.07),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()

  ability:activate(parent)
end
