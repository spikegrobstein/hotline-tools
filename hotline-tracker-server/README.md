# Hotline Tracker Server

Host your own [Hotline Tracker](https://hotline.fandom.com/wiki/Trackers) with this software.

> The name isn't final. But it's descriptive, right?

## Getting started

Well, first you gotta build it and put it somehwere you can run it.

```
cd hotline-tracker-server
cargo build --release
cp ../target/release/hotline-tracker-server /usr/local/bin
```

Then you can run it:

```
hotline-tracker-server start
```

or get help:

```console
$ hotline-tracker-server help
hotline-tracker-server

USAGE:
    hotline-tracker-server [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config <CONFIG>
            Path to config file

        --database <DATABASE>
            Override the path to the database file. By default this is adjacent
            to the config file or the current working directory if no config
            file

    -h, --help
            Print help information

SUBCOMMANDS:
    banlist     Add and remove servers from the banlist
    help        Print this message or the help of the given subcommand(s)
    password    Add and remove passwords to limit registrations
    start       Start the tracker server

```

By default, the server will create a database `tracker.sqlite3` in the current working directory when running
it. But you can control where this goes by using the `--database <db-path>` argument or by creating a config
file.

## Features

The Tracker server currently has the following features:

* Ability for servers to register with the tracker
* Clients can list the servers that are registered
* Only IPv4 connections are supported for server registration
* Registrations can be banned via the banlist
* Registrations can be restricted by requiring a password - with multiple accepted passwords so not every
    server uses the same credentials.

### The following (expected?) features are missing:

* No registration rate-limiting (any server can register as many times as it wants)
* No tracker listing is rate-limited (DoS attack is possible)

## Config file

The tracker supports a config file that can live at one of the following locations (in order of precedence):

* `--config <config-path>` CLI argument
* `TRACKER_CONFIG` environment variable
* `~/.hotline/tracker.toml`
* `~/.config/hotline/tracker.toml`
* `/etc/hotline/tracker.toml`

By default, the database will be stored in the same directory as the config file, if one is found, and will be
named `tracker.sqlite3`. This can be configured in the config file.

The config file is [TOML](https://toml.io) as the filename implies. The default config is equiivalent to:

```toml
[server]

# The address to bind all servers to (TCP port 5498 and UDP port 5499)
bind-address = 0.0.0.0

# require a password for servers to register
require-password = false

# path to the database (relative paths are relative to this file)
database = ./tracker.sqlite3
```

## Database

The database file is used to store the banlist and registration passwords. This makes it straight-forward to
update these lists without needing to reboot the server and allows external programs to easily query.

## Working with the banlist

Performing CRUD operations on the banlist is done through the `banlist` subcommand. Entries added to the
banlist can include notes to make it easy to remember _why_ an entry was added.

```console
$ hotline-tracker-server banlist --help
hotline-tracker-server-banlist
Add and remove servers from the banlist

USAGE:
    hotline-tracker-server banlist <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add       Add a server to the banlist by ipv4 address
    help      Print this message or the help of the given subcommand(s)
    list      List all servers in the banlist
    remove    Remove a server from the banlist
```

## Adding passwords

Working with passwords is done similarly to the banlist and also supports a notes field for including
arbitrary human-readable text about the entry.

```console
$ hotline-tracker-server password --help
hotline-tracker-server-password
Add and remove passwords to limit registrations

USAGE:
    hotline-tracker-server password <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add       Add an authorizzed password for server registrations
    help      Print this message or the help of the given subcommand(s)
    list      List all authorized passwords
    remove    Remove an authorized password
```

## Todo

There still some work to be done and this is a work in progress.

* Documentation (rustdoc)
* remove `unwrap()` calls and replace with actual errors
* DoS protection (rate limit registrations)
* Different interfaces for tracker and registration server?
* port config? (should this be supported)
* large response handling (updates in the middle)
* metrics
* cli log level?
* daemon? pidfile?
* listing options
* json output for lists
