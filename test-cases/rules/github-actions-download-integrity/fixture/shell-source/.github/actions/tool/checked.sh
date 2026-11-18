#!/bin/sh
curl -fsSL https://downloads.example.com/checked -o checked
sha256sum -c checked.sha256
