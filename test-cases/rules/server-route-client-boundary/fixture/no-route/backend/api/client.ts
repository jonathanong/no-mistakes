import axios from "axios";

export function loadUsers() {
  return axios.get("/users");
}
