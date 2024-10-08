# Melangerie

![Melangerie Screenshot](.\server\public\melangerie.png)

Melangerie is a real-time chat application built with Rust, Svelte, and Rocket, a web framework for Rust. The main purpose is an in-house alternative to apps like Microsoft Teams and Skype when just used for basic communication. It allows users to create chat rooms, invite other users to join, and send messages within those rooms. The application uses WebSockets for real-time communication and JSON Web Tokens (JWT) for user authentication.

## Features

- User registration and authentication using JWT
- Create and join chat rooms
- Real-time messaging within chat rooms
- Add users to existing chat rooms
- List available chat rooms for a user
- Offline message storage and retrieval upon user reconnection
- Reporting mechanism for users to report issues
- Logging of server events and user reports

## Project Structure

The project is structured as follows:

```
office-chat-rs/
├── server/
│   ├── public/
│   │   ├── index.html
│   │   └── ...
│   ├── src/
│   │   ├── auth.rs
│   │   ├── chat.rs
│   │   ├── cors.rs
│   │   ├── log.rs
│   │   ├── main.rs
│   │   ├── test.rs
│   │   └── types.rs
│   └── Cargo.toml
└── client/
    └── ...
```

- The `server` directory contains the Rust backend code.
  - `auth.rs`: Handles user registration, login, and JWT generation/verification.
  - `chat.rs`: Implements the WebSocket connection handling and chat room functionality.
  - `cors.rs`: Provides CORS support for the server.
  - `log.rs`: Defines the logging functionality for server events and user reports.
  - `main.rs`: The entry point of the application, setting up the Rocket server and routes.
  - `test.rs`: Contains unit tests for the application.
  - `types.rs`: Defines the data types used throughout the application.
- The `public` directory serves as the static file server, containing the HTML, CSS, and JavaScript files for the frontend.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:s
   ```
   git clone https://github.com/exotik850/melangerie.git
   ```

2. Change to the `client` directory and install dependencies:
    ```
    cd melangerie/client && npm i
    ```

3. Build the client for the backend to serve:
    ```
    npm run build
    ```
4. Change to the `server` directory:
   ```
   cd ../server
   ```

5. Set up the environment variables:
   - Create a `.env` file in the `server` directory.
   - Add the following variables to the `.env` file:
     ```
     JWT_SECRET=your-jwt-secret
     ```
     Replace `your-jwt-secret` with a secure secret key for JWT generation and verification.

6. Run the server:
   ```
   cargo run
   ```

   The server will start running at `http://0.0.0.0:8080`.

## API Endpoints

- `POST /auth/login`: User login endpoint. Expects a JSON payload with `name` and `password` fields.
- `POST /auth/createuser`: User registration endpoint. Expects a JSON payload with `name` and `password` fields.
- `GET /auth/checkuser/<name>`: Checks if a user with the given `name` exists.
- `GET /chat/connect`: WebSocket endpoint for establishing a chat connection.
- `GET /chat/list`: Lists the available chat rooms for the authenticated user.
- `POST /chat/adduser/<room>/<user_id>`: Adds a user with the given `user_id` to the specified `room`.
- `POST /chat/chatroom`: Sends a message to a chat room. Expects a JSON payload with `sender`, `room`, `content`, and `timestamp` fields.
- `POST /chat/create/<name>/<users..>`: Creates a new chat room with the given `name` and initial `users`.
- `GET /<file..>`: Serves static files from the `public` directory.
- `POST /report`: Endpoint for users to report issues. Expects a JSON payload with `name` and `issue` fields.

## Authentication

The application uses JSON Web Tokens (JWT) for user authentication. When a user logs in or registers, a JWT is generated and sent to the client. The client must include this token in the `Authorization` header for subsequent requests that require authentication.

## WebSocket Communication

The WebSocket endpoint (`/chat/connect`) handles real-time communication between the server and clients. Upon establishing a connection, the server authenticates the user using the provided JWT. Once authenticated, the user can join chat rooms, send messages, and receive messages from other users in real-time.

The WebSocket messages are JSON-encoded and follow a specific structure defined by the `UserAction` and `ServerAction` enums in `types.rs`.

## Logging

The application includes a logging mechanism to log server events and user reports. The `Log` struct in `log.rs` handles writing log messages to a file named `log.txt`. The server periodically flushes the log buffer to ensure that logs are persisted.

## Contributing

Contributions to the project are welcome! If you find any bugs or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).