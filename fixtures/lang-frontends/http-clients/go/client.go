package client

import "net/http"

func Load() {
    http.Get("http://localhost:8080/users")
}
