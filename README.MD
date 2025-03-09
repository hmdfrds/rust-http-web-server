# Rust HTTP Web Server

A minimal HTTP Web server written in Rust.

## Features

- **Static File Serving:** Serves files from the `www/` directory.
- **Directory Listing:** Generates a listing if no `index.html` is found.
- **HTTP Methods:** Supports GET and HEAD.
- **Admin Interface:** Accessible on the admin port with basic authentication (default: `admin`/`adminpass`).
- **Thread-Safe Logging:** Logs requests, errors, and periodic server statistics.

## Project Structure

```text
rust-http-web-server/ 
├── Cargo.toml 
├── config.json 
├── README.md 
├── www
|   └── index.html
└── src/ 
    ├── admin.rs 
    ├── config.rs 
    ├── handler.rs 
    ├── logger.rs 
    ├── server.rs 
    ├── utils.rs 
    └── main.rs
```

## Setup & Run

1. **Clone the repository:**

   ```bash
   git clone https://github.com/hmdfrds/rust-http-web-server.git
   cd rust-http-web-server
   ```

2. **Build the project:**

    ```bash
    cargo build --release
    ```

3. **Run the server:**

    ```bash
    cargo run --release
    ```

## Configuration

Edit `config.json` to adjust host, port, document root, and logging parameters.

## License

MIT License. See [License](LICENSE).
