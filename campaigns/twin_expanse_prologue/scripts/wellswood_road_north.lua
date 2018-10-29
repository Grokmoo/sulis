function on_enter_rockslide(parent, target)
  game:cancel_blocking_anims()
  game:toggle_prop_at(28, 104)

  game:show_cutscene("wellswood_road_north_rockslide")
end

function after_rockslide_cutscene(parent)
  game:scroll_view(35, 110)
  base_class = game:player():base_class()
  convo_started = false
  
  jorzal = game:entity_with_id("npc_jorzal")
  if base_class ~= "fighter" and jorzal:is_valid() then
    jorzal:teleport_to({x = 42, y = 113 })
	game:start_conversation("after_rockslide", jorzal)
	convo_started = true
  end
  
  aessa = game:entity_with_id("npc_aessa")
  if base_class ~= "mage" and aessa:is_valid() then
    aessa:teleport_to({ x = 35, y = 110})
	if not convo_started then
	  game:start_conversation("after_rockslide", aessa)
	  convo_started = true
	end
  end
  
  grazi = game:entity_with_id("npc_grazi")
  if base_class ~= "rogue" and grazi:is_valid() then
    grazi:teleport_to({x = 41, y = 109})
	if not convo_started then
	  game:start_conversation("after_rockslide", grazi)
	  convo_started = true
	end
  end
  
  game:set_quest_entry_state("the_goblin_trap", "rockslide", "Active")
end

function after_rockslide_dialog(parent)
  base_class = game:player():base_class()

  check_add_party_member("npc_jorzal", base_class)
  check_add_party_member("npc_grazi", base_class)
  check_add_party_member("npc_aessa", base_class)
end

function check_add_party_member(id, base_class)
  npc = game:entity_with_id(id)
  if not npc:is_valid() then return end
  
  if npc:base_class() == base_class then return end
  
  if not npc:is_party_member() then
    game:add_party_member(id)
  end
end