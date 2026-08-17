package httproutes

// Same handler names as the registration file so RouteRef can leave
// routes.go. net/http and mux helpers register the local wrapper.
func Health() {}

func Ready() {}

func Users() {}

func CreateItem() {}

func Ping() {}

func Status() {}
