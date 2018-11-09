function on_activate(parent, item)
  item:activate(parent)
  
  game:start_conversation("adventurers_note", parent)
end