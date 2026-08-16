<?php

require_once __DIR__ . '/../app/helpers.php';
use App\Contracts\Mailer;
use App\Jobs\{SomeJob as WelcomeJob};
use App\Http\Controllers\{PingController, UserController};
use App\Http\Controllers\UserController;

Route::get('/api/users', [UserController::class, 'index']);
Route::get('/ping', PingController::class);
Route::get('/fq-users', \App\Http\Controllers\UserController::class);
