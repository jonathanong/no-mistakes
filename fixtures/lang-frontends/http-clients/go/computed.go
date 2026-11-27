package computed

import (
    "fmt"
    "net/http"
)

func Load(id string) {
    http.Get(fmt.Sprintf("/users/%s", id))
}
