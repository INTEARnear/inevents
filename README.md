# inevents

`inevents` is a tool for building APIs for blockchain events.

## Features

- Redis streams: in [redis/](./redis/) there is a crate that you can use to `emit_event`, `read_event` for a single event, and `start_reading_events` for a continuous stream of events with continuation on interruption.
- Redis-to-database: Transform a Redis stream of events into a persistent PostgreSQL-like table.
- HTTP API for historical events: Coming soon.
- WebSocket API: `/events/<event_id>`, optionally send a JSON message containing the custom filter object.

## Getting started

Just add your own `Event` and run one of the modules (HttpServer, WebsocketServer, RedisToPostgres) or build your own one. Check the examples in [intear-events/](./intear-events/), they're used on `events.intear.tech` and are always up to date.
