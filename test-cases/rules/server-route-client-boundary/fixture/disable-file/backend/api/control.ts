import express from "express";
import axios from "axios";

const app = express();
const route = "/api/orders";
app.get("/api/orders", handler);

axios.get(route);
