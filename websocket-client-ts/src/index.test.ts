/// <reference types="jest" />
import { describe, expect, it, jest } from '@jest/globals';
import { EventStreamClient } from './index';

describe('EventStreamClient', () => {
    jest.setTimeout(15000); // 15 seconds

    it('should receive events from log_text stream', async () => {
        const client = EventStreamClient.default();
        const receivedEvents: unknown[] = [];

        await Promise.race([
            client.streamEvents<unknown>(
                'log_text',
                null,
                async (event) => {
                    receivedEvents.push(event);
                }
            ),
            new Promise(resolve => setTimeout(() => {
                client.abort();
                resolve(undefined);
            }, 10000))
        ]);

        expect(receivedEvents.length).toBeGreaterThan(0);
        console.log(`Received ${receivedEvents.length} events`);
        console.log('First event:', JSON.stringify(receivedEvents[0]));
        console.log('Last event:', JSON.stringify(receivedEvents[receivedEvents.length - 1]));
    });

    it('should filter events', async () => {
        const client = EventStreamClient.default();
        const receivedEvents: unknown[] = [];

        const filter = {
            And: [
                { path: 'event_standard', operator: { Equals: 'nep141' } }
            ]
        };

        await Promise.race([
            client.streamEvents<unknown>(
                'log_nep297',
                filter,
                async (event) => {
                    receivedEvents.push(event);
                }
            ),
            new Promise(resolve => setTimeout(() => {
                client.abort();
                resolve(undefined);
            }, 10000))
        ]);

        expect(receivedEvents.length).toBeGreaterThan(0);
        for (const event of receivedEvents) {
            expect((event as any).event_standard).toBe('nep141');
        }
        console.log(`Received ${receivedEvents.length} error events`);
    });
}); 