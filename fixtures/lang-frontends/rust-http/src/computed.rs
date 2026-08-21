pub fn register(path: &str, prefix: &str) {
    router.route(path, get(list_users));
    router.route("/x", get(a).post(b));
    web::resource(prefix).route(web::get().to(ready));
}

#[get(prefix)]
pub async fn hidden() {}
