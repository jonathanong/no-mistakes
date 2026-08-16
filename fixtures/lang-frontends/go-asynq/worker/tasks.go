package worker

import "github.com/hibiken/asynq"

func HandleWelcome(ctx interface{}, task *asynq.Task) error {
    return nil
}

func Register(mux *asynq.ServeMux) {
    mux.HandleFunc("mail:welcome", HandleWelcome)
}
