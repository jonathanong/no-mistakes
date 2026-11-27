package routes

import "net/http"

func Register() {
    http.HandleFunc("/users", ListUsers)
}

func ListUsers(w http.ResponseWriter, r *http.Request) {}
