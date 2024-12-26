import WebSocket from 'ws';

interface WebSocketAdapter {
    send(data: string, callback?: (err?: Error) => void): void;
    close(): void;
    terminate?(): void;
    on(event: string, listener: (...args: any[]) => void): void;
    once(event: string, listener: (...args: any[]) => void): void;
    addEventListener(event: string, listener: (...args: any[]) => void): void;
}

class NodeWebSocketAdapter implements WebSocketAdapter {
    private ws: WebSocket;

    constructor(url: string) {
        this.ws = new WebSocket(url);
    }

    send(data: string, callback?: (err?: Error) => void): void {
        this.ws.send(data, callback);
    }

    close(): void {
        this.ws.close();
    }

    terminate(): void {
        this.ws.terminate();
    }

    on(event: string, listener: (...args: any[]) => void): void {
        if (event === 'ping') {
            this.ws.on('ping', (data) => {
                this.ws.pong(data);
                listener(data);
            });
        } else {
            this.ws.on(event, listener);
        }
    }

    once(event: string, listener: (...args: any[]) => void): void {
        this.ws.once(event, listener);
    }

    addEventListener(event: string, listener: (...args: any[]) => void): void {
        this.ws.on(event, listener);
    }
}

class BrowserWebSocketAdapter implements WebSocketAdapter {
    private ws: globalThis.WebSocket;

    constructor(url: string) {
        this.ws = new globalThis.WebSocket(url);
    }

    send(data: string, callback?: (err?: Error) => void): void {
        try {
            this.ws.send(data);
            callback?.();
        } catch (err) {
            callback?.(err as Error);
        }
    }

    close(): void {
        this.ws.close();
    }

    terminate(): void {
        this.ws.close();
    }

    on(event: string, listener: (...args: any[]) => void): void {
        if (event === 'message') {
            this.ws.onmessage = (ev) => listener(ev.data);
        } else if (event === 'close') {
            this.ws.onclose = listener;
        } else if (event === 'error') {
            this.ws.onerror = listener;
        } else if (event === 'ping') {
            // Browser WebSocket API doesn't expose ping/pong
        }
    }

    once(event: string, listener: (...args: any[]) => void): void {
        const wrappedListener = (...args: any[]) => {
            this.ws.removeEventListener(event, wrappedListener);
            listener(...args);
        };
        if (event === 'message') {
            this.ws.addEventListener('message', (ev) => wrappedListener(ev.data));
        } else {
            this.ws.addEventListener(event, wrappedListener);
        }
    }

    addEventListener(event: string, listener: (...args: any[]) => void): void {
        if (event === 'message') {
            this.ws.addEventListener('message', (ev) => listener(ev.data));
        } else {
            this.ws.addEventListener(event, listener);
        }
    }
}

function createWebSocket(url: string): WebSocketAdapter {
    if (typeof window !== 'undefined' && window.WebSocket) {
        return new BrowserWebSocketAdapter(url);
    }
    return new NodeWebSocketAdapter(url);
}

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
            const ws = createWebSocket(url.toString());
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
                if (ws.terminate) {
                    ws.terminate();
                } else {
                    ws.close();
                }
                continue outer;
            }
        }
    }
}
