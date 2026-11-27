<?php

require_once __DIR__ . '/../app/helpers.php';
use App\Contracts\Mailer;
use App\Jobs\{SomeJob as WelcomeJob};
use App\Http\Controllers\{PingController, UserController};
use App\Http\Controllers\UserController;

Route::get('/api/users', [UserController::class, 'index']);
Route::get('/ping', PingController::class);
Route::get('/fq-users', \App\Http\Controllers\UserController::class);
Route::resource('users', UserController::class);
Route::resource('hidden', UserController::class, ['only' => ['index']]);
// Bare Route::apiResource is a route edge (collection + member), same as Route::resource.
Route::apiResource('accounts', UserController::class);
Route::resource('articles', UserController::class)->only(['show']);
Route::resource('labels', UserController::class)->names(['index' => 'labels.index'])->only(['index']);
Route::resource('photos.comments', PhotoCommentController::class);
Route::resource(name: 'named-users', controller: UserController::class, only: ['index', 'show']);
Route::apiResource(controller: UserController::class, name: 'named-accounts', except: ['destroy']);
Route::apiResources(['bulk' => UserController::class]);
Route::apiResource('filtered', UserController::class)->only(['index']);
Route::apiResource('excluded', UserController::class)->except(['index']);
Route::resource("$prefix/users", UserController::class);
Route::resource($name, UserController::class);
Route::resource(__DIR__ . '/files', UserController::class);
