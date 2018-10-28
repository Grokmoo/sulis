function on_enter_main_gate(parent)
  game:cancel_blocking_anims()
  game:scroll_view(98, 113)
  game:set_quest_entry_state("leader_of_beasts", "fortress_found", "Visible")
  game:start_conversation("swamp_fortress_main_gate", parent)
end

function on_enter_slave_pens(parent)
  target = game:entity_with_id("orc_slave_master")
  
  game:say_line("Fresh meat! Get them!", target)
end

function on_activate_irhan(parent)
  game:cancel_blocking_anims()
  game:scroll_view(20, 34)
  
  irhan = game:entity_with_id("irhan")
  irhan:teleport_to({x = 20, y = 34})
  
  game:start_conversation("irhan", parent)
end