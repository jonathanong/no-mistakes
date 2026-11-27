"""requests.get("http://localhost:8080/users")"""

import requests

def load(user_id, url):
    requests.get(url)
    requests.get(f"/users/{user_id}")
    requests.get("https://api.example.invalid/users?x=1")
    requests.get("//api.example.invalid/users")
    description = 'calls such as requests.get("http://localhost:8080/users")'
