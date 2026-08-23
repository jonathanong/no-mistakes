<?php

namespace App;

function computed($bus, $prefix, $message): void
{
    $bus->dispatch($message);
}

#[Route($prefix . '/computed')]
function computed_route(): void
{
}
