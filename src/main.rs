use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::signal;

mod handlers;
mod order;
mod restaurant;
mod table;

use restaurant::Restaurant;
use crate::handlers::{
    handle_post_order, 
    handle_get_order, 
    handle_delete_order
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AddOrderRequest {
    table_id: u32,
    items: Vec<u32>
}

/// Handles incoming connections.
///
/// Reads data from the stream, processes the request, and sends a response back.
/// If the request is invalid or an error occurs, it returns an appropriate error response.
async fn handle_connection(mut stream: TcpStream, restaurant: Restaurant) {
    let mut buffer = [0; 1024];
    if let Ok(n) = stream.read(&mut buffer).await {
        if n == 0 {
            return;
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let response = match handle_request(request.as_ref(), restaurant).await {
            Ok(response) => response,
            Err(err) => format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", err),
        };

        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("Error writing to stream: {}", e);
        }
    }
}

/// Parses the HTTP request, extracts the method and path, and handles the request.
///
/// Parameters:
/// - `request`: A string containing the HTTP request.
/// - `restaurant`: An instance of `Restaurant`.
///
/// Returns:
/// - `Ok(response)`: The HTTP response if successful.
/// - `Err(err)`: An error response if the request is invalid or an error occurs.
async fn handle_request(request: &str, restaurant: Restaurant) -> Result<String, String> {
    let lines: Vec<&str> = request.lines().collect();
    let method_path: Vec<&str> = lines[0].split_whitespace().collect();

    if method_path.len() != 3 {
        return Err("Invalid request".to_string());
    }

    let method = method_path[0];
    let path = method_path[1];

    match (method, path) {
        ("POST", "/orders") => {
            let response = match handle_post_order(request, restaurant).await {
                Ok(response) => response,
                Err(err) => format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", err)
            };
            Ok(response)
        }
        ("DELETE", path) if path.starts_with("/orders/") => {
            let response = match handle_delete_order(path, restaurant).await {
                Ok(response) => response,
                Err(err) => format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", err)
            };
            Ok(response)
        }
        ("GET", path) if path.starts_with("/orders/") => {
            let response = match handle_get_order(path, restaurant).await {
                Ok(response) => response,
                Err(err) => format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", err)
            };
            Ok(response)
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\nNot Found".to_string();
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(&addr).await.unwrap();

    let restaurant = Restaurant::new(150);

    println!("Server listening on: {}", addr);

    // Spawn a task to handle signals
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Ctrl+C received. Shutting down gracefully.");

        // Gracefully shutdown the server
        std::process::exit(0);
    });

    while let Ok((stream, _)) = listener.accept().await {
        let restaurant = restaurant.clone();
        // Spawning a new asynchronous task for each incoming connection
        tokio::spawn(handle_connection(stream, restaurant));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_valid_post_request() {
        let request = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": 6, \"items\": [101, 102]}";
        let restaurant = Restaurant::new(12); // Create a mock restaurant instance
        let result = handle_request(request, restaurant).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(), 
            "HTTP/1.1 200 OK\r\n\r\n{\"data\":\"{\\\"table_id\\\":6,\\\"items\\\":[101,102]}\",\"message\":\"Success!\",\"success\":true}"
        );
    }

    #[tokio::test]
    async fn test_invalid_request_line() {
        let request = "INVALID_REQUEST_LINE";
        let restaurant = Restaurant::new(12);
        let result = handle_request(request, restaurant).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid request");
    }

    #[tokio::test]
    async fn test_valid_delete_request() {
        let request = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": 15, \"items\": [16, 102]}";
        let restaurant = Restaurant::new(100); // Create a mock restaurant instance
        let restaurant2 = restaurant.clone();
        let restaurant3 = restaurant.clone();

        let _result = handle_request(request, restaurant).await;

        let request2 = "DELETE /orders/15/16 HTTP/1.1\r\n\r\n";
        let result2 = handle_request(request2, restaurant2).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "HTTP/1.1 200 OK\r\n\r\n{\"message\":\"Removed 16 from table 15\",\"success\":true}");

        let request3 = "DELETE /orders/10/16 HTTP/1.1\r\n\r\n";
        let result3 = handle_request(request3, restaurant3).await;
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "HTTP/1.1 400 Bad Request\r\n\r\n{\"message\":\"Order not found\",\"success\":false}");
    }

    #[tokio::test]
    async fn test_valid_get_request() {
        let request = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": 15, \"items\": [16, 102]}";
        let restaurant = Restaurant::new(100); // Create a mock restaurant instance
        let restaurant2 = restaurant.clone();
        let restaurant3 = restaurant.clone();

        let _result = handle_request(request, restaurant).await;

        // Get all orders
        let request2 = "GET /orders/15 HTTP/1.1\r\n\r\n";
        let result2 = handle_request(request2, restaurant2).await;
        assert!(result2.is_ok());
        let response = result2.unwrap();
        assert!(response.contains("\\\"item_id\\\":16,\\\"table_id\\\":15"));
        assert!(response.contains("\\\"item_id\\\":102,\\\"table_id\\\":15"));

        // Get 1 order
        let request3 = "GET /orders/15/items/16 HTTP/1.1\r\n\r\n";
        let result3 = handle_request(request3, restaurant3).await;
        assert!(result3.is_ok());
        let response2 = result3.unwrap();
        assert!(response2.contains("\\\"item_id\\\":16,\\\"table_id\\\":15"));
        assert!(!response2.contains("\\\"item_id\\\":102,\\\"table_id\\\":15"));
    }

    #[tokio::test]
    async fn test_invalid_request_path() {
        let request = "GET /invalid-path HTTP/1.1\r\n\r\n";
        let restaurant = Restaurant::new(100);
        let result = handle_request(request, restaurant).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HTTP/1.1 404 Not Found\r\n\r\nNot Found");
    }
}