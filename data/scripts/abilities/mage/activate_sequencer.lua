function on_activate(parent, ability)
  target_type = parent:get_flag("__sequencer_type")
  
  if target_type == "self" then
    fire_sequencer(parent, ability, parent)
	return
  elseif target_type == "friendly" then
    targets = parent:targets():friendly():reachable()
  elseif target_type == "hostile" then
    targets = parent:targets():hostile():visible_within(8)
  end
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  fire_sequencer(parent, ability, targets:first())
end

function fire_sequencer(parent, ability, target)
  ability:activate(parent)
  
  anim = parent:wait_anim(0.2)
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_complete_fn("fire_spell_1")
  anim:set_completion_callback(cb)
  anim:activate()
end

function fire_spell_1(parent, ability, targets)
  target = targets:first()
  spell_id_1 = parent:get_flag("__sequencer_spell_1")
  ability_to_fire = parent:get_ability(spell_id_1)
  use_spell(parent, ability_to_fire, target)
  
  anim = parent:wait_anim(0.5)
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_complete_fn("fire_spell_2")
  anim:set_completion_callback(cb)
  anim:activate()
end

function fire_spell_2(parent, ability, targets)
  target = targets:first()
  if not target:is_dead() then
    spell_id_2 = parent:get_flag("__sequencer_spell_2")
    ability_to_fire = parent:get_ability(spell_id_2)
    use_spell(parent, ability_to_fire, target)
  end
  
  seq_ability = parent:get_ability("spell_sequencer")
  seq_ability:deactivate(parent)
end

function use_spell(parent, spell, target)
  effect = parent:create_effect("Sequencer", 0)
  effect:add_num_bonus("ability_ap_cost", 10 * game:ap_display_factor())
  effect:add_free_ability_group_use()
  effect:apply()

  parent:use_ability(spell, true)
  
  if game:has_targeter() then
    if game:check_targeter_position(target:x(), target:y()) then
	  game:activate_targeter()
	else
	  game:cancel_targeter()
	  game:warn("Unable to fire sequencer spell for " .. spell:name())
	end
  end
end