<?php

use App\Contracts\Mailer;
use App\Jobs\{SomeJob as WelcomeJob};
use App\Http\Controllers\{UserController};
use App\Http\Controllers\UserController;

Route::get('/api/users', [UserController::class, 'index']);
