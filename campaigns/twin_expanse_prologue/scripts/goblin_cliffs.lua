function on_enter_ambush(parent, target)
  game:cancel_blocking_anims()
  game:spawn_encounter_at(83, 46)
  game:scroll_view(88, 55)
  game:start_conversation("goblin_cliffs_ambush", game:player())
end

function after_escape_cutscene(parent, target)
  local export = game:create_module_export("twin_expanse_act1")
  export:activate()
end

function on_area_load(parent)
  game:set_quest_entry_state("the_goblin_trap", "cliffs", "Active")
end

function enter_goblin_camp(parent)
  game:cancel_blocking_anims()
  game:scroll_view(41, 54)
  game:start_conversation("enter_goblin_camp", game:player())
end
