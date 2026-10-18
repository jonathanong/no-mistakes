// Both arms create the same sparse extra, but only one changes dense length.
import { query } from "@bench/db";
declare const choose: boolean;

const values = ["SELECT original", "SELECT retained"];
choose
  ? (values.push("SELECT appended"), (values[1000] = "SELECT sparse extra"))
  : (values[1000] = "SELECT sparse extra");
values[0] = "SELECT dense after shape change";
values[1001] = "SELECT sparse after shape change";
query(values[0]);
query(values[1001]);
query(values[1000]);

function sibling() {
  query("SELECT sibling baseline");
}
sibling();

query("SELECT root baseline");
