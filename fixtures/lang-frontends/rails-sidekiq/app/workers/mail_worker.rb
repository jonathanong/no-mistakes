class MailWorker
  include Sidekiq::Worker

  def perform(user_id)
  end
end
