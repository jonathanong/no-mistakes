<?php

function listen($consumer, $topic)
{
    $consumer->subscribe(["mail.welcome"]);
    $consumer->subscribe(["mail.$tenant"]);
    $consumer->subscribe([$topic]);
    $consumer->subscribeRegex("/^mail.*/");
}
