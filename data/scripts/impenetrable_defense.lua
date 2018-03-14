function on_activate()
  targets = parent:get_visible()
  game:log("on_activate from lua")
  for i = 1, #targets do
    game:log(tostring(targets[i]))
  end
end

function on_target_select()
  game:log("on target select")
end
