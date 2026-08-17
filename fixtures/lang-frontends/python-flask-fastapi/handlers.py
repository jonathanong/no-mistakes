# Same handler names as the decorator files so RouteRef can leave the
# decorator file. Flask/FastAPI decorate the local wrapper; the view lives here.
def users():
    return []


def ping():
    return "ok"


def health():
    return {"ok": True}


def create_item():
    return {}
