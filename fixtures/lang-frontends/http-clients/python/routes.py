from flask import Flask
from fastapi import FastAPI

app = Flask(__name__)
api = FastAPI()

@app.route("/users")
def users():
    return []

@api.get("/customers/{customer_id}")
def customer(customer_id: str):
    return {"id": customer_id}
