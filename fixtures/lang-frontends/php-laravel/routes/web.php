<?php

use App\Http\Controllers\{UserController};
use App\Http\Controllers\UserController;

Route::get('/api/users', [UserController::class, 'index']);
