def send_welcome(producer):
    producer.send("mail.welcome", value={"ok": True})
