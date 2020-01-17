-- removes all elements in the array where retain_fn returns false
function array_retain(data, retain_fn)
  local j = 1
  local size = #data
  
  for i = 1, size do
    if retain_fn(data, i) then
	  if i ~= j then
	    data[j] = data[i]
		data[i] = nil
      end
	  j = j + 1
	else
	  data[i] = nil
	end
  end
  
  return data
end