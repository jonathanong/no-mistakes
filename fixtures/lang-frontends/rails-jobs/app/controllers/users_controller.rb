class UsersController
  def index
    WelcomeJob.perform_later
  end
end
