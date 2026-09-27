import { it } from "vitest";
import { WebClient, loadPage } from "./client";
import { WebClient as OtherClient, loadPage as otherLoad } from "./client.js";

// Both spellings resolve to one export; mutation through either invalidates it.
OtherClient.prototype.request = async () => {};
otherLoad = async () => {};
const client = new WebClient();

it("aliased exports and reassigned receivers cannot claim routes", () => {
  client.request("/healthz");
  loadPage("", "/healthz");
});
