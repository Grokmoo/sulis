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

function boss_death(player)
  local boss = game:entity_with_id("berkeley_final")
  local max_hp = boss:stats().max_hp
  boss:take_damage(player, max_hp, max_hp, "Raw", 1000)
  
  game:scroll_view(boss:x(), boss:y())
  game:block_ui(2.0)
  game:run_script_delayed("xandala", "boss_death2", 2.0)
end

function boss_death2(player)
  game:block_ui(2.0)

  local targets = player:targets():hostile():to_table()
  for i = 1, #targets do
    targets[i]:remove()
  end
  
  game:run_script_delayed("xandala", "show_cutscene", 2.0)
end

function show_cutscene(player)
  game:show_cutscene("campaign_end")
end

function on_cutscene_end(player)
  game:exit_to_menu()
end