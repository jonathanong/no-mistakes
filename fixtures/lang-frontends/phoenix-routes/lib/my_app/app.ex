defmodule MyApp.App do
  alias MyApp.User

  def run, do: User.list()
end
