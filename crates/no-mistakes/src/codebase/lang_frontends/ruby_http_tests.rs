use super::*;
use crate::codebase::lang_frontends::strip::strip_comments_keep_strings;

#[test]
fn to_routes_and_bare_resources_expand() {
    let source = r#"
  get "/api/users", to: "users#index"
  resources :users
  resources "posts"
"#;
    let routes = extract_routes(source);
    assert!(routes.contains(&("/api/users".into(), "users#index".into())));
    assert!(routes.contains(&("/users".into(), "users#index".into())));
    assert!(routes.contains(&("/users/:id".into(), "users#show".into())));
    assert!(routes.contains(&("/users".into(), "users#create".into())));
    assert!(routes.contains(&("/users/:id".into(), "users#update".into())));
    assert!(routes.contains(&("/users/:id".into(), "users#destroy".into())));
    assert!(routes.contains(&("/posts".into(), "posts#index".into())));
}

#[test]
fn namespaced_only_except_and_singular_resource_are_non_edges() {
    let source = r#"
  resources :hidden, only: [:index]
  resources :skipped, except: [:destroy]
  resource :profile
  namespace :admin do
    resources :users
  end
  resources name
"#;
    assert!(extract_routes(source).is_empty());
}

#[test]
fn shallow_indented_namespaced_resources_are_non_edges() {
    let source = "namespace :admin do\n  resources :users\nend\nresources :posts\n";
    let routes = extract_routes(source);
    assert!(routes.iter().all(|(path, _)| path != "/users"));
    assert!(routes.contains(&("/posts".into(), "posts#index".into())));
    assert!(extract_routes("namespace \"admin\" do\n  resources :users\nend\nend\n").is_empty());
}

#[test]
fn comment_resources_are_not_routes() {
    let source = strip_comments_keep_strings(
        r#"
# resources :hidden
  resources :users
"#,
    );
    let routes = extract_routes(&source);
    assert!(routes.contains(&("/users".into(), "users#index".into())));
    assert!(routes.iter().all(|(path, _)| path != "/hidden"));
}
