<?php

namespace App\Http\Controllers;

use App\Jobs\SomeJob;

class UserController
{
    public function index()
    {
        SomeJob::dispatch();
    }
}
