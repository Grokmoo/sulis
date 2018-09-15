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
end
