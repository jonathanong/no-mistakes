import express from "express";

const router = express.Router();

router.get("/admin/:id", handler);

export default router;
