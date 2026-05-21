import axios from "axios";
import express from "express";
import auth from "./auth";

const app = express();

app.use(auth);
axios.get("/api/users");
