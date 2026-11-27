pub fn load(id: &str) {
    reqwest::get(format!("/users/{}", id));
}
