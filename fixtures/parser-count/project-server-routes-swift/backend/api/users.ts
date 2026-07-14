import express from "express";
import adminRouter from "./admin-router";

const app = express();

app.get("/api/users/:id", handler);
app.use("/api", adminRouter);
