package pkg

import "testing"

func TestPing(t *testing.T) {
	if Ping() != "pong" {
		t.Fatal("ping")
	}
}
