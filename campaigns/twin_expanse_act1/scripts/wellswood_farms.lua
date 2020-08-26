function on_area_load(parent)
  local target = game:entity_with_id("gethruk")
  game:start_conversation("farms_intro", target)
end

function gethruk_leave_intro(parent)
  local target = game:entity_with_id("gethruk")
  if not target:move_towards_point(25, 52) then
    game:log("Gethruk unable to move")
  end

  game:run_script_delayed("wellswood_farms", "gethruk_moved_finish_intro", 2.0)
end

function gethruk_moved_finish_intro(parent)
 local  target = game:entity_with_id("gethruk")
  target:remove()
  
  game:set_quest_entry_state("the_thug", "start", "Visible")
  game:player():set_flag("the_thug_active")
end

function gethruk_investigated(parent)
  game:set_quest_entry_state("the_thug", "investigated", "Visible")
end

function rockslide_investigated(parent)
  game:set_quest_entry_state("the_rockslide", "investigated", "Visible")
end

function adventurer_complete(parent)
  game:add_party_xp(50)
  game:set_quest_entry_state("a_rosy_picture", "complete", "Visible")
  game:set_quest_state("a_rosy_picture", "Complete")
end

function adventurer_talked(parent)
  game:set_quest_entry_state("a_rosy_picture", "start", "Visible")
end

function about_to_exit(parent)
  game:cancel_blocking_anims()
  game:scroll_view(106, 56)
  game:start_conversation("wellswood_farms_about_to_exit", parent)
end

function vaalyun_quest_start(parent)
  game:set_quest_entry_state("vaalyuns_journey", "start", "Visible")
end

function vaalyun_join(parent)
  game:add_party_member("npc_vaalyun")
end