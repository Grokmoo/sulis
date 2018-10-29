function on_activate(parent, item)
  item:activate(parent)
  
  game:start_conversation("troll_journal", parent)
end