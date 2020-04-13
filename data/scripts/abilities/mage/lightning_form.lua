function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  local cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local anim = parent:create_anim("central_lightning")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-1.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(2.0))
  anim:set_draw_above_entities()
  effect:add_anim(anim)
  
  local anim = parent:create_anim("central_lightning")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-1.0), anim:param(-2.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(2.0))
  anim:set_draw_below_entities()
  effect:add_anim(anim)

  effect:add_resistance(100, "Shock")
  
  effect:apply()
  ability:activate(parent)
  
  parent:add_ability("lightning_leap")
  
  game:play_sfx("sfx/warp3")
  game:play_sfx("sfx/ElectricityDamage01")
end

function on_removed(parent)
  parent:remove_ability("lightning_leap")
end