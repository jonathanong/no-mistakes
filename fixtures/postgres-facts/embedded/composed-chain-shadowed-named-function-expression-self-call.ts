import { query } from "@data-stores/psql";

function build() {
  return "SELECT 1";
}

const wrapper = function build() {
  return query(build());
};

export { wrapper };
