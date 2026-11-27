import requests

def load():
    requests.get("http://localhost:8080/users")
    requests.get("http://localhost:8080/shared/client-check")
    requests.get("http://localhost:8080/customers/jane%20doe")
    requests.get("http://localhost:8080/members/42")
