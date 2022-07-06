<!--
SPDX-FileCopyrightText: 2022 Thomas Kramer

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# LibrEDA DB

LibrEDA DB is a collection of interface definitions and data structures for chip layouts and netlists.

## Documentation

To view the documentation of this library in a browser clone this repository
and run `cargo doc --open`.

Alternatively a possible outdated version is hosted [here](https://libreda.org/doc/libreda_db/index.html) or [here](https://libreda.codeberg.page/doc/libreda_db/index.html).

## Current state

Most important functionality for handling layouts and netlists is already there.
But this is still WORK IN PROGRESS and not stable yet.

## Known shortcomings & ideas for future work

* [ ] Provide a way to check if an ID is valid. For example with non-panicking `.try_*() -> Option<*>` functions.
* [ ] Power domains: There's not a good way yet to represent power domains.
* [ ] Region search: Implement region search as a decorator for LayoutEdit/LayoutBase traits.
* [ ] Modification observer: Implement a decorator which allows to observe modifications on database structures using callback functions.
