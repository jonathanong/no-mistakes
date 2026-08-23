defmodule MyAppWeb.Computed do
  resources "/users", UserController
  get path, UserController, :index
end
