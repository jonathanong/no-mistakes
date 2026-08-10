import express from "express";

const app = express();
app.get("/users/:id", handler);

export { app as users };
