import express from "express";

const app = express();
app.get("/admin/users", (_req, res) => {
  res.json([]);
});
