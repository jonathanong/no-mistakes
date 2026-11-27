pub fn load() {
    reqwest::get("http://localhost:8080/users");
}
