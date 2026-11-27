defmodule MyApp.Client do
  def load do
    Req.get("/users")
  end
end
