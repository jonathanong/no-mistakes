#!/bin/sh
# integrity-check: skip reason=vendor
curl https://downloads.example.invalid/source-first
curl https://downloads.example.invalid/source-second
curl https://downloads.example.invalid/source-third # integrity-check: skip reason=vendor
curl https://downloads.example.invalid/source-fourth
