<?php

namespace App\MessageHandler;

use App\Message\WelcomeMessage;
use Symfony\Component\Messenger\Attribute\AsMessageHandler;

#[AsMessageHandler]
class WelcomeHandler
{
    public function __invoke(WelcomeMessage $message): void
    {
    }
}
