defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
end
