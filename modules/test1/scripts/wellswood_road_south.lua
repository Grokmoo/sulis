function on_add_aessa(parent, target)
  game:add_party_member("npc_aessa")
end

function on_add_jorzal(parent, target)
  game:add_party_member("npc_jorzal")
end

function on_add_grazi(parent, target)
  game:add_party_member("npc_grazi")
end

function on_player_enter_bridge(parent, target)
  game:spawn_encounter_at(17, 36)
  game:enable_trigger_at(35, 66)
end

function on_player_return(parent, target)
  game:spawn_encounter_at(49, 81)
end

function on_area_load(parent)
  target = game:entity_with_id("npc_tervald")
  game:start_conversation("tervald", target)
end

function on_ambush_cleared(parent)
  target = game:entity_with_id("npc_tervald")
  target:set_flag("ambush_cleared")
end
