function on_activate(parent, ability)
  game:log("Parent: " .. parent:to_string())
  
  targets = parent:targets():hostile():attackable():collect()
  
  for i = 1, #targets do
    game:log("Target at " .. i .. ": " .. targets[i]:to_string())
  end
end

function on_target_select()
  game:log("on target select")
end
