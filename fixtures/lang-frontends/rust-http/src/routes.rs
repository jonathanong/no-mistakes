use crate::handlers::{create_item, list_users, ready};

pub fn register(router: Router) -> Router {
    router
        .route("/users", get(list_users))
        .route("/items", post(create_item));
    web::resource("/ready").route(web::get().to(ready));
    router
}
