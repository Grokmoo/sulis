function on_activate(parent, ability)
  game:log("Parent: " .. parent:to_string())
  
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("defense", 20)
  effect:add_num_bonus("reflex", 10)
  effect:add_num_bonus("fortitude", 10)
  effect:add_num_bonus("will", 10)
  effect:apply()
  
  ability:activate(parent)
end
