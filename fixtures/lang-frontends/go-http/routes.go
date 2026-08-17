package httproutes

import "net/http"

func Register(mux *http.ServeMux) {
	http.HandleFunc("/health", Health)
	mux.Handle("/ready", Ready)
	r.Get("/users", Users)
	g.POST("/items", CreateItem)
	e.PUT("/ping", Ping)
	app.Delete("/status", Status)
}
