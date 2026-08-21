class UsersController
  def index
    MailWorker.perform_async(user.id)
    Workers::DigestJob.perform_async
  end
end
