import express from "express";

const app = express();

// This route must stay excluded by the configured test-file filter.
app.get("/api/test-only", handler);
