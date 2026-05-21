// guardrails-disable-file server-route-client-boundary
import express from "express";
import axios from "axios";

const app = express();
const route = "/api/users";

app.get("/api/users", handler);
axios.get(route);
