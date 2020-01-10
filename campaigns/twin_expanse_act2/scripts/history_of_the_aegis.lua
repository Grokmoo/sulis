function on_activate(parent, item)
  item:activate(parent)
  
  game:start_conversation("history_of_the_aegis", parent)
end