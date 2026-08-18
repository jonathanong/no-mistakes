package httproutes

import "net/http"

func TestRegister(mux *http.ServeMux) {
	http.HandleFunc("/from-test", Health)
}
