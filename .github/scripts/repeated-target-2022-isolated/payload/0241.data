import { query } from "@example/db";

function build() {
  return "SELECT 1";
}

const wrapper = function build() {
  return query(build());
};

export { wrapper };
