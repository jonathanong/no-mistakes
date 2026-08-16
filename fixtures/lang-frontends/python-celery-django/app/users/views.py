from . import models
from .models import User
from app.tasks import *

def user_list():
    return User

class UserView:
    pass
