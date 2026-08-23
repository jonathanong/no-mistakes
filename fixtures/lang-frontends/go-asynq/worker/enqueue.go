package worker

import fmtlib "fmt"

import (
	"example.com/nested"
	"github.com/hibiken/asynq"
)

func EnqueueWelcome(client *asynq.Client) error {
	_, err := client.Enqueue(asynq.NewTask("mail:welcome", nil))
	if err != nil {
		return fmtlib.Errorf("enqueue: %w", err)
	}
	return nil
}
