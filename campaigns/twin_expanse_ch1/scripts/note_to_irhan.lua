function on_activate(parent, item)
  item:activate(parent)
  
  game:set_quest_entry_state("leader_of_beasts", "leader_defeated", "Visible")
  game:start_conversation("note_to_irhan", parent)
  game:player():set_flag("leader_of_beasts_defeated")
end