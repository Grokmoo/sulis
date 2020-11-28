-- OnDamaged Hook
function berkeley_on_damaged(parent, targets, hit)
    local target = targets:first()
	
	game:log(parent:name() .. " damaged by " .. target:name() .. ": "
	    .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
end

-- OnRoundElapsed Hook
function berkeley_on_round_elapsed(parent)
  parent:set_flag("new_round")
end

function ai_action_berkeley(parent, params)
  if parent:has_flag("new_round") then
    local last_pillar_count = parent:get_num_flag("pillar_count")
  
    local pillar_count = pillar_count()

    if pillar_count > 0 then  
	  parent:heal_damage(parent:stats().max_hp)
    end
  
    if pillar_count < last_pillar_count then
      game:say_line(pick_line(), parent)
    else
      game:say_line("I am invincible!", parent)
    end
  
    parent:clear_flag("pillar_count")
    parent:add_num_flag("pillar_count", pillar_count)
  
    parent:clear_flag("new_round")
  end
  
  return ai_action(parent, params)
end

function pillar_count()
  local count = 2

  if game:entity_with_id("summoning_pillar_01"):is_dead() then count = count - 1 end
  if game:entity_with_id("summoning_pillar_02"):is_dead() then count = count - 1 end

  return count
end

function pick_line()
  local options = {
    "You are truly lost.",
	"How dare you try to interfere.",
	"Your time is at an end.",
	"I am a god!",
	"Why fight the inevitable?"
  }

  return options[math.random(#options)]
end

--INCLUDE ai_basic