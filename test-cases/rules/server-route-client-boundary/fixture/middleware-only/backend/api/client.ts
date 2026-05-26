import axios from "axios";
import express from "express";

const app = express();

app.use(auth);
axios.get("/api/users");

function auth() {}
