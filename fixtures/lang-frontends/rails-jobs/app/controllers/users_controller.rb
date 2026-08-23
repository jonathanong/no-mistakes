require "app/jobs/welcome_job"
require_relative "../jobs/welcome_job"
require_relative "/missing/outside"

class UsersController
  def index
    WelcomeJob.perform_later
    Admin::User
  end
end
