require_relative "../jobs/welcome_job"

class UsersController
  def index
    WelcomeJob.perform_later
  end
end
