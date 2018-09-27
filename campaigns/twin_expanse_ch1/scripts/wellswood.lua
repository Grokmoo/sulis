function goblin_raids_start(parent)
  game:set_quest_entry_state("leader_of_beasts", "start", "Visible")
end

function goblin_raids_leads(parent)
  game:set_quest_entry_state("leader_of_beasts", "leads", "Visible")
end

function cragnik_join(parent)
  game:add_party_member("npc_cragnik")
end

function enter_square(parent)
  game:cancel_blocking_anims()
  game:scroll_view(98, 47)
  game:start_conversation("wellswood_enter_square", parent)
end