package routes

import "net/http"

func Register() {
    http.HandleFunc("/users", ListUsers)
    http.HandleFunc("/shared/client-check", ListUsers)
}

func ListUsers(w http.ResponseWriter, r *http.Request) {}
