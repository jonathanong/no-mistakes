from app.views import user_list


def test_user_list():
    assert user_list() is not None
