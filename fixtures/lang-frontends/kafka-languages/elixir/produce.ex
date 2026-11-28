defmodule Mail.Publish do
  def send(client, topic, key, value) do
    :brod.produce_sync(client, "mail.welcome", :hash, key, value)
    :brod.produce_sync(client, topic, :hash, key, value)
  end
end
