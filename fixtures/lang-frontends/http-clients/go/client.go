package client

import (
    "context"
    "net/http"
)

func Load(ctx context.Context) {
    http.Get("http://localhost:8080/users")
    http.NewRequestWithContext(ctx, http.MethodGet, "/plain-context", nil)
    http.NewRequestWithContext(context.WithValue(ctx, "GET", "/wrong"), http.MethodGet, "http://localhost/nested-context", nil)
}
