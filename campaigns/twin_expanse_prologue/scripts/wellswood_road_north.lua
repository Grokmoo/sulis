function on_enter_rockslide(parent, target)
  game:cancel_blocking_anims()
  game:toggle_prop_at(28, 104)

  game:show_cutscene("wellswood_road_north_rockslide")
end

function after_rockslide_cutscene(parent)
  game:scroll_view(35, 110)

  local members = game:entities_with_ids({"npc_jorzal", "npc_aessa", "npc_grazi" })
  for i = 1, #members do
    game:remove_party_member(members[i]:id())
  end
  
  local base_class = game:player():base_class()
  if base_class == "fighter" then
    members[1]:remove()
    table.remove(members, 1)
  elseif base_class == "rogue" then
    members[3]:remove()
    table.remove(members, 3)
  else -- mage or druid
    members[2]:remove()
    table.remove(members, 2)
  end
  
  local positions = { {x=42, y=113},
                      {x=35, y=110},
					  {x=41, y=109}}
  local convo_started = false
  for i = 1, #members do
    member = members[i]
	member:teleport_to(positions[i])
	
	if not convo_started then
	  game:start_conversation("after_rockslide", member)
	  convo_started = true
	end
	
	game:add_party_member(member:id())
  end
  
  game:set_quest_entry_state("the_goblin_trap", "rockslide", "Active")
end