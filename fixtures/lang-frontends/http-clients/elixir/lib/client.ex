defmodule MyApp.Client do
  def load do
    Req.get("http://localhost:8080/users")
  end
end
