function on_activate(parent, ability)
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(12.0)
  targeter:set_free_select(12.0)
  targeter:set_free_select_must_be_passable("2by2")
  targeter:set_shape_object_size("2by2")
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  ability:activate(parent)
  
  local summon = game:spawn_actor_at("phantasm", pos.x, pos.y, "Neutral")
  if not summon:is_valid() then return end
  
  summon:set_flag("__phantasm_faction", parent:get_faction())
  
  local stats = parent:stats()
  
  local effect = summon:create_effect(ability:name(), ability:duration())
  effect:add_abilities_disabled()
  effect:add_attack_disabled()
  effect:add_move_disabled()
  effect:add_resistance(100, "Slashing")
  effect:add_resistance(100, "Crushing")
  effect:add_resistance(100, "Piercing")
  effect:add_num_bonus("spell_accuracy", stats.spell_accuracy)
  effect:add_num_bonus("caster_level", stats.caster_level)
  
  local cb = ability:create_callback(summon)
  cb:set_on_round_elapsed_fn("on_round_elapsed")
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local anim = summon:create_color_anim()
  anim:set_color(anim:param(0.01), anim:param(0.01), anim:param(0.01), anim:param(1.0))
  effect:add_color_anim(anim)
  
  effect:apply()
  
  check_terrify(summon, ability)
end

function on_round_elapsed(parent, ability)
  check_terrify(parent, ability)
end

function check_terrify(parent, ability)
  local faction = parent:get_flag("__phantasm_faction")
  local targets = parent:targets():hostile_to(faction):visible_within(8)
  
  local targets = targets:to_table()
  for i = 1, #targets do
    local target = targets[i]
    local hit = parent:special_attack(target, "Will", "Spell")
    local duration = 2
    if hit:is_miss() then
      duration = 0
    elseif hit:is_graze() then
      duration = duration - 1
    elseif hit:is_hit() then
      -- do nothing
    elseif hit:is_crit() then
      duration = duration + 1
    end
    
	if duration > 0 then
      local effect = target:create_effect(ability:name(), duration)
      effect:set_tag("fear")
      effect:add_attack_disabled()
      effect:add_num_bonus("will", -20)
      
      local anim = target:create_color_anim()
      anim:set_color(anim:param(0.8),
                     anim:param(0.1),
                     anim:param(0.1),
                     anim:param(1.0))
      anim:set_color_sec(anim:param(0.3),
                         anim:param(0.0),
                         anim:param(0.0),
                         anim:param(0.0))
      effect:add_color_anim(anim)
      effect:apply()
	  end
  end
end

function on_removed(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  local anim = parent:create_color_anim(1.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(1.0, -1.0))
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:set_completion_callback(cb)
  anim:activate()
end

function on_remove_complete(parent, ability)
  parent:remove()
end