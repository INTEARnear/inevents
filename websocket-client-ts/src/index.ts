import WebSocket from 'ws';

export class EventStreamClient {
    private url: URL;
    private abortController: AbortController;

    constructor(url?: string) {
        this.url = new URL(url ?? 'wss://ws-events-v3.intear.tech');
        this.abortController = new AbortController();
    }

    static default(): EventStreamClient {
        return new EventStreamClient();
    }

    abort(): void {
        this.abortController.abort();
    }

    private async delay(ms: number): Promise<void> {
        if (!this.abortController.signal.aborted) {
            await new Promise(resolve => setTimeout(resolve, ms));
        }
    }

    async streamEvents<E>(
        eventId: string,
        callback: (event: E) => Promise<void>
    ): Promise<void> {
        outer: while (!this.abortController.signal.aborted) {
            const url = new URL(`events/${eventId}`, this.url);
            const ws = new WebSocket(url.toString());
            let isAborting = false;

            try {
                await new Promise<void>((resolve, reject) => {
                    this.abortController.signal.addEventListener('abort', () => {
                        isAborting = true;
                        ws.close();
                        resolve();
                    });

                    ws.once('open', () => resolve());
                    ws.once('error', (error) => reject(error));
                });

                if (this.abortController.signal.aborted) {
                    break outer;
                }

                await new Promise<void>((resolve, reject) => {
                    ws.send('{"And":[]}', (error) => {
                        if (error) reject(error);
                        else resolve();
                    });
                });

                await new Promise<void>((resolve, reject) => {
                    this.abortController.signal.addEventListener('abort', () => {
                        isAborting = true;
                        ws.close();
                        resolve();
                    });

                    ws.on('message', async (data) => {
                        try {
                            const events: E[] = JSON.parse(data.toString());
                            for (const event of events) {
                                if (this.abortController.signal.aborted) {
                                    resolve();
                                    return;
                                }
                                await callback(event);
                            }
                        } catch (error) {
                            if (!isAborting) {
                                console.warn('Failed to parse or process message:', error);
                                reject(error);
                            }
                        }
                    });

                    ws.on('ping', (data) => {
                        ws.pong(data);
                    });

                    ws.on('close', () => {
                        if (!isAborting) {
                            console.warn(`Event stream events/${eventId} closed, reconnecting in 1 second`);
                            reject(new Error('WebSocket closed'));
                        }
                    });

                    ws.on('error', (error) => {
                        if (!isAborting) {
                            console.warn(`WebSocket error:`, error);
                            reject(error);
                        }
                    });
                });
            } catch (error: unknown) {
                if (this.abortController.signal.aborted) {
                    break outer;
                }
                if (!isAborting) {
                    console.warn(`Reconnecting to event stream events/${eventId} in 1 second`);
                    await this.delay(1000);
                }
                ws.terminate();
                continue outer;
            }
        }
    }
}

// Example usage and test
if (require.main === module) {
    (async () => {
        const client = EventStreamClient.default();
        const receivedEvents: unknown[] = [];

        try {
            await Promise.race([
                client.streamEvents<unknown>(
                    'log_text',
                    async (event) => {
                        console.log('Received event:', JSON.stringify(event));
                        receivedEvents.push(event);
                    }
                ),
                new Promise(resolve => setTimeout(() => {
                    client.abort();
                    resolve(undefined);
                }, 10000))
            ]);
        } catch (error: unknown) {
            console.error('Unexpected error:', error);
        }
    })();
} 