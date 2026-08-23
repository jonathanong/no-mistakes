Rails.application.routes.draw do
  get "/api/users", to: "users#index"
  get "/admin/users", to: "admin/users#index"
  resources :users
  # only:/except:, singular resource, and namespaced resources stay non-edges.
  resources :hidden, only: [:index]
  resource :profile
  namespace :admin do
    resources :accounts
  end
end
