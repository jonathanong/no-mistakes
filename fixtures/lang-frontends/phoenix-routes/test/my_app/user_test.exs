defmodule MyApp.UserTest do
  alias MyApp.User
  get "/phantom", MyAppWeb.UserController, :index
end
