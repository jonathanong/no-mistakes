from app.tasks import send_welcome


def test_send_welcome():
    assert send_welcome is not None
