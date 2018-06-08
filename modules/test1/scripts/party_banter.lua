names = { "npc_aessa", "npc_grazi", "npc_jorzal" }

function on_enter_wellswood_road_cave(parent, target)
  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("I doubt there is another way out of this cave, but I suppose it is worth a look...", party_member)
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