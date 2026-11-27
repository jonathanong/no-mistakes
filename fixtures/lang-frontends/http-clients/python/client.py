import requests

def load():
    requests.get("/users")
    requests.get("https://example.test/users?x=1")
