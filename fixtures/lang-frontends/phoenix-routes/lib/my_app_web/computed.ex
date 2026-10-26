defmodule MyAppWeb.Computed do
  resources "/users", UserController
  get path, UserController, :index
  scope path do
    get "/hidden", UserController, :index
  end

  # Interpolated prefixes are computed paths, not literal joins.
  scope "/x#{v}" do
    get "/c", UserController, :index
  end
end
