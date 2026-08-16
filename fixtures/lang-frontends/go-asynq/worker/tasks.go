package worker

import (
	"context"

	"github.com/hibiken/asynq"
)

type WelcomePayload struct{}

func HandleWelcome(ctx context.Context, task *asynq.Task) error {
	return nil
}

func Register(mux *asynq.ServeMux) {
    mux.HandleFunc("mail:welcome", HandleWelcome)
}
