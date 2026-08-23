defmodule App.Run do
  alias Shared.User

  def run, do: User.list()
end
