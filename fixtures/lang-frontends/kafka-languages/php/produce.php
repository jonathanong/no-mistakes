<?php

function publish($producer, $topic)
{
    $producer->newTopic("mail.welcome");
    $producer->newTopic($topic);
}
