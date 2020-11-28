function on_cave_load(player)
  game:run_script_delayed("campaign", "heal_party", 0.0)
  game:say_line("The magic in the air instantly heals your wounds and recovers your abilities.", game:player())
end

function boss_init(player)
  local boss = game:entity_with_id("berkeley_final")

  game:set_quest_entry_state("the_aegis", "final_battle", "Visible")
  game:set_quest_state("the_aegis", "Complete")


  game:scroll_view(53, 10)
  boss:add_num_flag("pillar_count", 2)
  game:start_conversation("berkeley_final", boss)
  boss:set_faction("Hostile")
  game:spawn_encounter_at(29, 7)
end