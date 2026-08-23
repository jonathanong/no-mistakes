import express from "express";

const app = express();
app.get("/api/users", (_req, res) => {
  res.json([]);
});
