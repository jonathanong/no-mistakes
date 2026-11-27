<?php

function listen($consumer, $topic)
{
    $consumer->subscribe(["mail.welcome"]);
    $consumer->subscribe([$topic]);
    $consumer->subscribeRegex("/^mail.*/");
}
