package worker

import "github.com/hibiken/asynq"

func EnqueueWelcome(client *asynq.Client) error {
	_, err := client.Enqueue(asynq.NewTask("mail:welcome", nil))
	return err
}
