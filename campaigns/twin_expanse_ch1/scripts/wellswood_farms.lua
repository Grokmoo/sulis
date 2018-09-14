function on_area_load(parent)
  target = game:entity_with_id("farmer")
  game:start_conversation("farms_intro", target)
end

function gethruk_leave_intro(parent)
  target = game:entity_with_id("gethruk")
  if not target:move_towards_point(24, 50) then
    game:log("Gethruk unable to move")
  end
  
  cb = game:create_callback(target, "wellswood_farms")
  cb:set_on_anim_complete_fn("gethruk_moved_finish_intro")
  anim = target:wait_anim(2.0)
  anim:set_completion_callback(cb)
  anim:activate()
end

function gethruk_moved_finish_intro(parent)
  parent:remove()
end