package httproutes

import "net/http"

func Register(mux *http.ServeMux) {
	http.HandleFunc("/health", Health)
}
