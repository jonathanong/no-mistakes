new Date().toLocaleString("en-US");
new Date().toLocaleDateString("en-GB", { dateStyle: "short" });
new Date().toLocaleTimeString(["fr-FR", "en-US"]);
new Date()?.toLocaleString?.("en-US");
new Date()["toLocaleDateString"](["en-GB"]);
new Date()[`toLocaleTimeString`]("en-US");
function formatDate(date) {
  return date.toISOString();
}
formatDate(new Date());
function toLocaleString(date) {
  return date.toISOString();
}
toLocaleString(new Date());
