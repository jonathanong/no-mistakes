defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
  post "/users", MyAppWeb.UserController, :create

  # Literal scope prefixes join onto nested verb paths; computed `scope path` is a non-edge.
  scope "/api" do
    get "/items", MyAppWeb.UserController, :index
  end
end
