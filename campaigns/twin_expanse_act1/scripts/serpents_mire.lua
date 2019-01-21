function encounter_01_spawn(parent)
  game:cancel_blocking_anims()
  game:scroll_view(102, 12)
  
  spawn_target_at("slime", 95, 10)
  spawn_target_at("slime_blue", 96, 15)
  spawn_target_at("slime", 107, 14)
end

function encounter_02_spawn(parent)
  game:cancel_blocking_anims()
  game:scroll_view(98, 103)
  
  spawn_target_at("slime", 92, 100)
  spawn_target_at("slime_orange", 92, 104)
  spawn_target_at("slime", 105, 104)
  spawn_target_at("slime_red", 110, 109)
end

function encounter_03_spawn(parent)
  game:cancel_blocking_anims()
  game:scroll_view(43, 57)
  
  spawn_target_at("slime", 36, 54)
  spawn_target_at("slime_blue", 45, 57)
  spawn_target_at("slime", 38, 61)
  spawn_target_at("slime_red", 52, 58)
end

function spawn_target_at(target_id, x, y)
  local entity = game:spawn_actor_at(target_id, x, y)
  if not entity:is_valid() then return end
  
  local hide = entity:create_color_anim(1.0)
  hide:set_color(hide:param(1.0),
                 hide:param(1.0),
                 hide:param(1.0),
                 hide:param(0.0, 1.0))
  hide:activate()
  
  local anim = entity:create_anim("dust_cloud", 0.65)
  anim:set_position(anim:param(x + entity:width() / 2 - 2), anim:param(y + entity:height() / 2 - 2))
  anim:set_particle_size_dist(anim:fixed_dist(4.0), anim:fixed_dist(4.0))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.5))
  anim:set_draw_above_entities()
  anim:set_blocking(false)
  anim:activate()
end