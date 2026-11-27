defmodule MyApp.Computed do
  def load(path) do
    Req.get(path)
  end
end
