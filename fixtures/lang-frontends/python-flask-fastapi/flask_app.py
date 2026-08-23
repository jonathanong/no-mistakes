from flask import Flask, Blueprint

app = Flask(__name__)
bp = Blueprint("api", __name__)


@app.route("/users")
def users():
    return []


@bp.get("/ping")
def ping():
    return "ok"
