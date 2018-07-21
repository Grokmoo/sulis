function on_activate(parent, ability)
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("armor", 15)
  effect:add_attribute_bonus("Dexterity", -4)

  anim = parent:create_color_anim()
  anim:set_color(anim:param(0.2),
                 anim:param(0.15),
                 anim:param(0.0),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.3),
                     anim:param(0.2),
                     anim:param(0.0),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()

  ability:activate(parent)
end
