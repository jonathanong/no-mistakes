// no-mistakes-disable-file server-route-client-boundary
import express from "express";
import axios from "axios";

const app = express();
const route = "/api/widgets";
app.get("/api/widgets", handler);

axios.get(route);
