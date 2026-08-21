class UsersController
  def index
    MailWorker.perform_async(user.id)
    DigestJob.perform_async
  end
end
