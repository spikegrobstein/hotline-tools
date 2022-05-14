# Hotline tools

This repo contains some libraries and tools for working with hotline.

For the most part:

* `hotline-bookmark-cli` -- a commandline tool for reading and writing hotline bookmark files
* `hotline-tracker-client` -- commandline tracker client -- list servers on a tracker + register your server
    with a tracker
* `hotline-tracker-server` -- run your own tracker server.

this is pretty much all still a work in progress. but things do function.

Additional docs forthcoming.

## Hotline bookmark cli

    cd hotline-bookmark-cli
    cargo run -- --help

also

    cargo run -- create --help
    cargo run -- print --help

## Hotline tracker client

    cd hotline-tracker-client
    cargo run -- --help

Also:

    cargo run -- list --help
    cargo run -- register --help

## Hotline tracker server

    cd hotline-tracker-server
    cargo run

> there isn't any costomization yet. all settings are hard-coded but it works.
