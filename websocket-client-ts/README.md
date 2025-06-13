# inevents-websocket-client

Event streaming client for Intear events API. This client allows you to subscribe to and process events from the Intear WebSocket API with automatic reconnection and error handling.

## Installation

```bash
npm install @intear/inevents-websocket-client
```

## Usage

```typescript
import { EventStreamClient } from '@intear/inevents-websocket-client';

// Create a client
const client = EventStreamClient.default();

// Start streaming events
await client.streamEvents<YourEventType>(
    'event_name',
    null, /* or { And: [ { ... filters ... } ] } */
    async (event) => {
        // Handle each event
        console.log('Received event:', event);
    }
);

// To stop streaming
client.abort();
```

### Custom Server URL

```typescript
const client = new EventStreamClient('wss://your-server.com');
```

### Type-Safe Events

The client is fully type-safe. Define your event types for better TypeScript integration:

```typescript
interface MyEvent {
    event_type: string;
    data: Record<string, unknown>;
}

await client.streamEvents<MyEvent>(
    'my_event',
    async (event) => {
        // event is typed as MyEvent
        console.log(event.event_type);
    }
);
```

## Features

- Type-safe event streaming
- Automatic reconnection on connection loss
- Proper error handling
- Clean shutdown support
- WebSocket ping/pong handling
- Configurable server URL

## License

MIT 

