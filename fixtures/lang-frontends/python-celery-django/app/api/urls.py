from django.urls import path
from app.users import views

urlpatterns = [
    path("nested/", views.user_list),
]
