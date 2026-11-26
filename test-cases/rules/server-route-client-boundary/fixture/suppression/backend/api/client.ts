import express from "express";
import axios from "axios";

const app = express();
const route = "/api/widgets";
app.get("/api/widgets", handler);

axios.get(route); // no-mistakes-disable-line server-route-client-boundary
// no-mistakes-disable-next-line server-route-client-boundary
axios.get(route);
axios.get(route);
