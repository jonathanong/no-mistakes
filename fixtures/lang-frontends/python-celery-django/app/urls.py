from django.urls import path
from app.users import views

urlpatterns = [
    path("api/users/", views.user_list),
    path("users/", views.UserView.as_view()),
]
