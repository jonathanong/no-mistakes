from app.users import User, create_user


def test_create_user():
    assert create_user() is not None
    assert User
