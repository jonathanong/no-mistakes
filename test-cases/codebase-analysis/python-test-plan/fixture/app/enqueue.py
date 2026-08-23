from app.tasks import send_welcome


def invite(user_id: int) -> None:
    send_welcome.delay(user_id)
