# Simple Restaurant API Server

## Build and Run

You need to have Rust and Cargo installed.

Run build command:

```
$ cargo build
```

Run the server:

```
$ cargo run
```

Run the unittests:

```
$ cargo test
```

## Test with Clients

Run the server

```
$ cargo run
```

Open another shell and run the client:

```
$ python3 client/client.py
```

## API Design

- `POST /orders/:table_id`: send order request with payload that contains order data object. Order data object contains array of items and a table id
- `DELETE /orders/:table_id/:item_id` delete an ordered item in a table
- `GET /orders/:table_id/items/:item_id`: get a specific ordered item in a table
- `GET /orders/:table_id`: show all items in a table

## License

MIT
