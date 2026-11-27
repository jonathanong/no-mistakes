pub fn register(router: Router) -> Router {
    router.route("/users", get(list_users))
}

fn list_users() {}
