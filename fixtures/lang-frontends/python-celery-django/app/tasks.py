from celery import shared_task

@shared_task(name="mail.send_welcome")
def send_welcome(user_id: int) -> None:
    return None
