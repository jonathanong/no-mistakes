import express from "express";

const app = express();
app.get("/users/:id", handler);
app.get("/local/:id", handler);
app.get("/imported/:id", handler);

export { app as users };
