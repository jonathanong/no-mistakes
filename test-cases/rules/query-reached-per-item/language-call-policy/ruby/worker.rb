def fetch(item)
  example.store.read_row(item)
end

def load(items)
  for item in items do
    fetch(item)
    item.fetch()
  end
end
