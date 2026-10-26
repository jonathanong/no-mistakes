defmodule MyApp.App do
  # Brace aliases expand to exact module imports; nested braces stay non-edges.
  alias MyApp.{User, Role}

  # Next-line tuples must not turn the exact alias into a brace group.
  alias MyApp.User
  {:ok, _x} = {:ok, 1}
  alias MyApp.Role
  {Foo, Bar} = {:ok, 1}

  # A semicolon ends the alias statement before another expression.
  alias MyApp.{User}; foo()

  # Extra closing brace is not a brace alias.
  alias MyApp.{Ghost}}

  # The outer brace is intentionally missing: reject the nested body before
  # accepting GhostNested as an otherwise valid comma-separated member.
  alias MyApp.{Foo.{Bar, GhostNested}

  def run, do: User.list()
end
