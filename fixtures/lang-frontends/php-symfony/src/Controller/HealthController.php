<?php

namespace App\Controller;

use Symfony\Component\Routing\Attribute\Route;

#[Route('/health', methods: ['GET'])]
class HealthController
{
    public function __invoke(): void
    {
    }
}
