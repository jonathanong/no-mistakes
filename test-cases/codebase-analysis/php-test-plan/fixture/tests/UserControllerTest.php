<?php

use App\Http\Controllers\UserController;

class UserControllerTest extends TestCase
{
    public function test_index(): void
    {
        $this->assertNotNull(UserController::class);
    }
}
