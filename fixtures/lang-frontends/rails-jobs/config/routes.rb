Rails.application.routes.draw do
  get "/api/users", to: "users#index"
  get "/admin/users", to: "admin/users#index"
end
