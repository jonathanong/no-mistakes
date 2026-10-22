defmodule MyAppWeb.Computed do
  resources "/users", UserController
  get path, UserController, :index
  scope path do
    get "/hidden", UserController, :index
  end
end
