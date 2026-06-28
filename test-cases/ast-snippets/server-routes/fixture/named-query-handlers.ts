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

export const exportedList = function (req, res) {
  req.query.exported;
  res.send("ok");
};

function delegated(req, res) {
  list(req, res);
  res.send("ok");
}

function destructured({ query: { namedParam } }, res) {
  res.send("ok");
}

export default function defaulted(req, res) {
  req.query.defaulted;
  res.send("ok");
}

app.get("/search", search);
app.get("/list", list);
app.get("/delegated", delegated);
app.get("/destructured", destructured);
app.get("/inline-destructured", ({ query: { inlineParam } }, res) => {
  res.send("ok");
});
app.get("/exported", exportedList);
app.get("/defaulted", defaulted);
