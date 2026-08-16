package worker

import "fmt"

import (
	"github.com/hibiken/asynq"
)

func EnqueueWelcome(client *asynq.Client) error {
	_, err := client.Enqueue(asynq.NewTask("mail:welcome", nil))
	if err != nil {
		return fmt.Errorf("enqueue: %w", err)
	}
	return nil
}
