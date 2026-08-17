<?php

namespace App;

use App\Message\WelcomeMessage;
use Symfony\Component\Messenger\MessageBusInterface;

function enqueue(MessageBusInterface $bus): void
{
    $bus->dispatch(new WelcomeMessage());
}
