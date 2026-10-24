import express from "express";

const app = express();

app.get("/api/users", handler);
