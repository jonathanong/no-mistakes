import requests

def load(user_id, url):
    requests.get(url)
    requests.get(f"/users/{user_id}")
