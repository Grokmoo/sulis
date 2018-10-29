names = { "npc_vaalyun", "npc_cragnik" }

function wellswood_forest_enter(parent, target)
  party_member = game:entity_with_id("npc_vaalyun")
  if party_member == nil then return end
  
  game:say_line("Ah, the forest.  Feels like home...", party_member)
end

function get_party_member()
  targets = game:entities_with_ids(names)
  
  if #targets == 0 then
    return nil
  else
    -- random party member
    return targets[math.random(#targets)]
  end
end