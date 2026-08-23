package worker

import "testing"

func TestHandleWelcome(t *testing.T) {
	if HandleWelcome == nil {
		t.Fatal("missing handler")
	}
}
