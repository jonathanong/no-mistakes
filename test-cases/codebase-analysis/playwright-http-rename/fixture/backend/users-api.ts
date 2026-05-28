import { client } from "./client";

export async function listUsersV2() {
  return client.get("/api/v2/users");
}
