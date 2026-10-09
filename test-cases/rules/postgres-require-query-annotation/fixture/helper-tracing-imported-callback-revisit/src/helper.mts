import { write } from "@app/db";
function create(statement: string) {
  return () => {
    const saved = arguments[0];
    const assigned = (arguments[0] = "SELECT 1");
    write(saved); // finding:imported-revisit
  };
}
export const send = create("/* imported initial */ SELECT 1");
