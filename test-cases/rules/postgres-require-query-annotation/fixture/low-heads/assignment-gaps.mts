import { read } from "@bench/db";

function load() {
  let query = "/* first */ SELECT 1";
  query = "/* reassigned */ SELECT 2";
  read(query);
}
