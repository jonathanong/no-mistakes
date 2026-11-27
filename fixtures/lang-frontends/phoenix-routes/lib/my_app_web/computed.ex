defmodule MyAppWeb.Computed do
  get path, UserController, :index
  scope path do
    get "/hidden", UserController, :index
    resources "/users", UserController
  end

  # Interpolated prefixes are computed paths, not literal joins.
  scope "/x#{v}" do
    get "/c", UserController, :index
  end
end
