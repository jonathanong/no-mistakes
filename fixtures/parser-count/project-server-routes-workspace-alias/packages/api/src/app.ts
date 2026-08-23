import express from "express";
import adminRouter from "@routers/admin-router";

const app = express();

app.use("/api", adminRouter);
