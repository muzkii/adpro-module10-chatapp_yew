import WebSocket, { WebSocketServer } from 'ws';

const PORT = process.env.PORT ? parseInt(process.env.PORT) : 8080;
interface User {
    ws: WebSocket;
    nick: string;
    isAlive: boolean;
}

interface Message {
    messageType: string;
    data: string;
    dataArray: string[];
}

let users: User[] = [];

console.log(`Listening on port ${PORT}`);
const wss = new WebSocketServer({ port: PORT });

wss.on('connection', (ws: WebSocket) => {
    console.log('ws connected');

    ws.on('message', (data) => {
        const raw_data = data.toString();
        try {
            const parsed_data: Message = JSON.parse(raw_data);
            switch (parsed_data.messageType) {
                case 'register':
                    users.push({ ws, nick: parsed_data.data, isAlive: true });
                    broadcast(JSON.stringify({ messageType: 'users', dataArray: users.map((u) => u.nick) }));
                    break;
                case 'message':
                    const sender = users.find((u) => u.ws === ws);
                    if (sender) {
                        broadcast(
                            JSON.stringify({
                                messageType: 'message',
                                data: JSON.stringify({
                                    from: sender.nick,
                                    message: parsed_data.data,
                                    time: Date.now(),
                                }),
                            })
                        );
                    }
                    break;
                case 'reaction':
                    {
                        const sender = users.find((u) => u.ws === ws);
                        // parsed_data.data: JSON.stringify([messageIndex, emoji])
                        const [messageIndex, emoji] = JSON.parse(parsed_data.data);
                        broadcast(JSON.stringify({
                            messageType: 'reaction',
                            data: JSON.stringify({ messageIndex, emoji, from: sender?.nick }),
                        }));
                    }
                    break;
                case 'readReceipt':
                    {
                        const sender = users.find((u) => u.ws === ws);
                        // parsed_data.data: messageIndex
                        broadcast(JSON.stringify({
                            messageType: 'readReceipt',
                            data: JSON.stringify({ messageIndex: parsed_data.data, user: sender?.nick }),
                        }));
                    }
                    break;
            }
        } catch (e) {
            console.log('Error in message', e);
        }
    });
});

const interval = setInterval(function ping() {
    const current_clients = Array.from(wss.clients);
    const updated_users = users.filter((u) => current_clients.includes(u.ws));
    if (updated_users.length !== users.length) {
        users = updated_users;
        broadcast(JSON.stringify({ messageType: 'users', dataArray: users.map((u) => u.nick) }));
    }
}, 5000);

const broadcast = (data: any) => {
    wss.clients.forEach((client) => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(data);
        }
    });
};
