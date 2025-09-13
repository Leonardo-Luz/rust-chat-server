# Rust Chat Server

*A simple WebSocket chat server built with Rust, Tokio, and Tungstenite. This server supports multiple chat rooms, user-defined nicknames and colors, and broadcasts messages as JSON objects, including the number of clients in the room.*

## Features

*   **WebSocket Communication**: Real-time communication using WebSockets.
*   **Multiple Chat Rooms**: Users can join different chat rooms.
*   **User Nicknames**: Users can set their nicknames upon connecting.
*   **Customizable Colors**: Users can set a hex color for their nicknames.
*   **JSON Message Format**: All messages are broadcast as JSON, including sender, content, room, color, and client count.
*   **Room Passwords**: Rooms can be protected with a password.

## Installation

To get started with the chat server, you need to have Rust and Cargo installed. If you don't have them, you can install them using `rustup`,

Once Rust is installed, clone the repository:

```bash
git clone https://github.com/leonardo-luz/chat-server.git
cd chat-server
```

## Running the Server

To run the server, simply use Cargo:

```bash
cargo run
```

By default, the server will listen on `127.0.0.1:9001`.

## Usage

You can connect to the WebSocket server using any WebSocket client, but the `leonardo-luz/rust-chat-client-tui` repository was specifically made for this.

### Message Format

All messages received from the server will be in the following JSON format:

```json
{
  "msg_type": "chat" | "info" | "error",
  "sender": "nickname" | "server",
  "color": "RRGGBB",
  "content": "Your message here",
  "room": "room_name",
  "client_count": 5
}
```

*   `msg_type`: Indicates the type of message (e.g., "chat" for user messages, "info" for server notifications, "error" for error messages).
*   `sender`: The nickname of the sender or "server" for server-generated messages.
*   `color`: The hex color of the sender's nickname.
*   `content`: The actual message content.
*   `room`: The room the message belongs to.
*   `client_count`: The number of clients currently in the `room`.

## Commands

Users can send special commands by typing them in the chat:

* `/join <room_name> [password]`: Joins a specified room. If the room doesn't exist, it will be created. If a password is provided, the room will be protected.
* `/color <hex_color>`: Changes the user nickname color

## Contributing

Contributions are welcome! If you have any suggestions, bug reports, or want to improve the server, feel free to open an issue or submit a pull request.
