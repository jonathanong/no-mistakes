import { query } from "@example/db";

function build() {
  return "SELECT 1";
}

const wrapper = function build() {
  return build();
};

export function run() {
  return query(wrapper());
}
