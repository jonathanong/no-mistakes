<?php
function fetch($item) {
    example\store::read_row($item);
}

function load($items) {
    foreach ($items as $item) {
        fetch($item);
        $item->fetch();
    }
}
