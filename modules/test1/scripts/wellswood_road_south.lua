function on_talk_tervald(parent, target)
  game:add_party_member("npc_aessa")
end

function on_player_enter_bridge(parent, target)
  game:log("Enter bridge")
  game:spawn_encounter_at(47, 78)
  game:spawn_encounter_at(16, 47)
end

function on_area_load(parent)
  target = game:entity_with_id("npc_tervald")
  game:start_conversation("tervald", target)
end

function on_ambush_cleared(parent)
  target = game:entity_with_id("npc_tervald")
  target:set_flag("ambush_cleared")
end
