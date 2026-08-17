package httproutes

import "net/http"

func RegisterComputed(pattern string) {
	http.Handle(pattern, Health)
}
