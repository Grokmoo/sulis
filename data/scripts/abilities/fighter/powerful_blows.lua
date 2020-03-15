function on_activate(parent, ability)
  local cur_mode = parent:get_active_mode()
  if cur_mode ~= nil then
    cur_mode:deactivate(parent)
  end

  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  local stats = parent:stats()
  
  effect:add_num_bonus("melee_accuracy", 10 + stats.level / 2)
  effect:add_num_bonus("defense", -10)
 
  if parent:ability_level(ability) > 1 then
    effect:add_damage(3, 8 + stats.level / 2, 5)
	effect:add_num_bonus("crit_multiplier", 0.5)
  else
    effect:add_damage(2, 5 + stats.level / 2, 0)
  end
  
  local cb = ability:create_callback(parent)
  cb:set_on_held_changed_fn("on_held_changed")
  effect:add_callback(cb)

  local gen = parent:create_anim("crossed_swords")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  game:play_sfx("sfx/sword_sheath_1")
  ability:activate(parent)
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function on_held_changed(parent, ability)
  if not parent:stats().attack_is_melee then
    game:say_line("Powerful Blows Deactivated", parent)
    ability:deactivate(parent)
  end
end
