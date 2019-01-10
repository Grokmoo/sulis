function on_activate(parent, item)
  item:activate(parent)
  
  game:start_conversation("aegis_staff", parent)
end