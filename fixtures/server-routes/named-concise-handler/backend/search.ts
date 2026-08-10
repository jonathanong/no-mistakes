import express from "express";

const app = express();

// Oxc 0.143 represents this concise handler body as an expression, not statements.
const handler = req => req.query.term;

app.get("/search", handler);
