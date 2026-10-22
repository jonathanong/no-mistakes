defmodule MyApp.App do
  # Brace aliases expand to exact module imports; nested braces stay non-edges.
  alias MyApp.{User, Role}

  def run, do: User.list()
end
