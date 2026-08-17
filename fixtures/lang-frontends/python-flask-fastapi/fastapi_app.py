from fastapi import FastAPI, APIRouter

app = FastAPI()
router = APIRouter()


@app.get("/health")
async def health():
    return {"ok": True}


@router.post("/items")
def create_item():
    return {}
