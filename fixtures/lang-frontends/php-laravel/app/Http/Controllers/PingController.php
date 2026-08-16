<?php

namespace App\Http\Controllers;

class PingController
{
    public function __invoke()
    {
        return 'ok';
    }
}
