Rails.application.routes.draw do
  get "/api/users", to: "users#index"
end
