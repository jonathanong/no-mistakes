<?php

function publish($producer, $topic)
{
    $producer->newTopic("mail.welcome");
    $producer->newTopic("mail.$tenant");
    $producer->newTopic($topic);
}
