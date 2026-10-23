// Every excluded file deliberately contains a reportable cache directive.
export async function cached() {
  "use cache";
  return import("./provider");
}
