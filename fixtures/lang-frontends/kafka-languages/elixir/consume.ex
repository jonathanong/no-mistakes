defmodule Mail.Listen do
  def listen(client, topic) do
    :brod.subscribe(client, self(), "mail.welcome", [])
    :brod.subscribe(client, self(), topic, [])
    :brod.subscribe(client, self(), ~r/mail.*/, [])
  end
end
