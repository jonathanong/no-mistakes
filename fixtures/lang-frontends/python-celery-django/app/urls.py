from django.urls import include, path
from app.billing import views as billing_views
from app.users import views

urlpatterns = [
    path("api/", include("app.api.urls")),
    path("api/users/", views.user_list),
    path("users/", views.UserView.as_view()),
    path("users-index/", views.index),
    path("billing-index/", billing_views.index),
]
