defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
  post "/users", MyAppWeb.UserController, :create
end
