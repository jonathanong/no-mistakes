import express from "express";

const app = express();

function search(req, res) {
  req.query.term;
  res.send("ok");
}

const list = (req, res) => {
  req.query.page;
  res.send("ok");
};

function delegated(req, res) {
  list(req, res);
  res.send("ok");
}

app.get("/search", search);
app.get("/list", list);
app.get("/delegated", delegated);
