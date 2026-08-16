from . import models
from .models import User
from app.tasks import *

"""
class LegacyUser:
    pass
"""

def user_list():
    return User

def index():
    return User

class UserView:
    pass
