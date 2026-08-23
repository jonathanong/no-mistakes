class UsersController < ApplicationController
  def index
    WelcomeJob.perform_later
  end
end
