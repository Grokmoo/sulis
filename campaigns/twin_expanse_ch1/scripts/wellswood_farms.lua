function on_area_load(parent)
  target = game:entity_with_id("farmer")
  game:start_conversation("farms_intro", target)
end

function gethruk_leave_intro(parent)
  target = game:entity_with_id("gethruk")
  if not target:move_towards_point(24, 50) then
    game:log("Gethruk unable to move")
  end

  game:run_script_delayed("wellswood_farms", "gethruk_moved_finish_intro", 2.0)
end

function gethruk_moved_finish_intro(parent)
  target = game:entity_with_id("gethruk")
  target:remove()
  
  game:set_quest_entry_state("the_thug", "start", "Visible")
end

function gethruk_investigated(parent)
  game:set_quest_entry_state("the_thug", "investigated", "Visible")
end

function rockslide_investigated(parent)
  game:set_quest_entry_state("the_rockslide", "investigated", "Visible")
end

function adventurer_talked(parent)
  game:set_quest_entry_state("a_rosy_picture", "start", "Visible")
end