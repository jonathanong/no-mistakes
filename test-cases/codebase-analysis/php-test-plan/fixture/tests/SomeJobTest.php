<?php

use App\Jobs\SomeJob;

class SomeJobTest extends TestCase
{
    public function test_job(): void
    {
        $this->assertNotNull(SomeJob::class);
    }
}
