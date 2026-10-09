const database = require("@app/db");

function sloppySlotQuery() {
  const slots = arguments;
  slots[0] = "SELECT 4";
  database.query(slots[0]); // finding:sloppy-argument-alias
}

sloppySlotQuery();
