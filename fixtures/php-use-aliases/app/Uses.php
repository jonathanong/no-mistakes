<?php

namespace App;

use App\Jobs\SomeJob as Job;
use App\Dto\{UserDto as Dto, Missing};

final class Uses
{
    public function make(): Job
    {
        return new Job(new Dto());
    }
}
