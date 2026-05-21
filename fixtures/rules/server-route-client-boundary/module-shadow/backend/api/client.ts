import realAxios from "axios";
import axios from "axios";
import express from "express";

const app = express();

axios.get("/module-shadow-local-only");

const axios = realAxios;

app.get("/users", () => {});
axios.get("/api/users");
