function on_enter_main_gate(parent)
  game:cancel_blocking_anims()
  game:scroll_view(98, 113)
  game:set_quest_entry_state("leader_of_beasts", "fortress_found", "Visible")
  game:start_conversation("swamp_fortress_main_gate", parent)
end
