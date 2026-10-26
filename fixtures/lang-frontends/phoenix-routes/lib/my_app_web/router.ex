defmodule MyAppWeb.Router do
  get "/users", MyAppWeb.UserController, :index
  post "/users", MyAppWeb.UserController, :create

  # Literal scope prefixes join onto nested verb paths; computed `scope path` is a non-edge.
  scope "/api" do
    get "/items", MyAppWeb.UserController, :index
  end # api

  # Trailing-comment `end` must pop so later verbs stay unprefixed.
  get "/after-comment-end", MyAppWeb.UserController, :index

  # One-line keyword-list scope must not leak onto later verbs (`get(` stays a non-edge).
  scope "/e", do: get("/f", MyAppWeb.UserController, :index)
  get "/z", MyAppWeb.UserController, :index

  # Same-line `do ... end` joins without leaving the prefix on the stack.
  scope "/g" do get "/h", MyAppWeb.UserController, :index end
  get "/after-inline", MyAppWeb.UserController, :index

  # Trailing module argument still opens a literal prefix.
  scope "/", MyAppWeb do
    get "/aliased", MyAppWeb.UserController, :index
  end
end
