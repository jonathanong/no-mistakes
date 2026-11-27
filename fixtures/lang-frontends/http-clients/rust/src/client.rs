pub fn load() {
    reqwest::get("/users");
}
