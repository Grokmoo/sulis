function on_talk_tervald(parent, target)
  --game:log("hello, world")
end

function on_player_enter_bridge(parent, target)
  game:log("Enter bridge")
  game:spawn_encounter_at(46, 72)
  game:spawn_encounter_at(16, 47)
end

function on_area_load(parent)
  target = game:entity_with_id("npc_tervald")
  game:start_conversation("tervald", target)
end
