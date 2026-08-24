import express from "express";

const app = express();

app.get("/api/v1/feeds/rss_feed_items/:feedType", handler);
app.get("/api/v1/feeds/posts/:feedType", handler);
