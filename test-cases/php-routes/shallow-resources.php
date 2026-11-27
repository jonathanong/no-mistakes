<?php
Route::resource('photos.comments', CommentController::class)->shallow();
Route::apiResource('albums.photos', PhotoController::class)-> shallow (true);
Route::apiResources(['photos.comments' => CommentController::class])->shallow();
