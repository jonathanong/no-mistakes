from django.urls import path
from app.views import user_list

urlpatterns = [
    path("users/", user_list),
]
