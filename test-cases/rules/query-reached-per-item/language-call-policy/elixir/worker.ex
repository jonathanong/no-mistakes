defmodule Worker do
  def fetch(item) do
    example.store.read_row(item)
  end

  def load(items) do
    for item <- items do
      fetch(item)
      item.fetch()
    end
  end
end
