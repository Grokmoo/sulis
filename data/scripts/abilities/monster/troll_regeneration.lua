function on_activate(parent, ability)
  local stats = parent:stats()
  parent:heal_damage(8 + stats.level)
  ability:activate(parent)
end
