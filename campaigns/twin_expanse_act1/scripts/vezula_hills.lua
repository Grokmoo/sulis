function find_rockslide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(60, 80)
  game:start_conversation("vezula_hills_rockslide", parent)
end

function rockslide_final_fate(parent)
  game:set_quest_entry_state("the_rockslide", "final_fate", "Visible")
  game:set_quest_state("the_rockslide", "Complete")
  game:add_party_xp(100)
end

function kaelwyn_leave(parent)
  local target = game:entity_with_id("kaelwyn")
  target:remove()
end

function learn_ring_gone(parent)
  game:set_quest_entry_state("vaalyuns_journey", "ring_gone", "Visible")
  game:add_party_xp(50)
end

function enable_worldmap(parent)
  game:set_world_map_location_visible("vezula_hills", true)
  game:set_world_map_location_enabled("vezula_hills", true)
end

function activate_troll_encounter(parent)
  local troll = game:entity_with_id("forest_troll_leader")
  
  game:say_line("Up and at em, boys!  Kills these fools then its back to the mire!", troll)
end

function serpent_mire_map(parent)
  game:set_quest_entry_state("leader_of_beasts", "serpents_mire", "Visible")
  game:add_party_xp(50)
  
  game:set_world_map_location_visible("serpents_mire", true)
  game:set_world_map_location_enabled("serpents_mire", true)
end