pub fn load() {
    let _ = reqwest::Client::new()
        .get("/chained");
    reqwest::get("http://localhost:8080/users");
}
