from flask import Flask

app = Flask(__name__)

@app.route("/users")
def users():
    return []
