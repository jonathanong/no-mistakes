package client

import "net/http"

func Load() {
    http.Get("/users")
}
