use crate::{AddOrderRequest, Restaurant};
use serde_json;
use serde_json::json;

/// Handles a POST request for adding an order.
///
/// # Arguments
///
/// * `request`: A string containing the HTTP request.
/// * `restaurant`: The restaurant instance.
///
/// # Returns
///
/// Returns a `Result` with either an HTTP response or an error message.
pub async fn handle_post_order(
    request: &str,
    restaurant: Restaurant,
) -> Result<String, String> {
    let body_start = request.find("\r\n\r\n").ok_or("Invalid request")? + 4;
    let body = &request[body_start..];

    let order_request: AddOrderRequest = match serde_json::from_str(body) {
        Ok(request) => request,
        Err(err) => {
            let response = json!({
                "success": false,
                "message": format!("Failed to parse order request: {}", err)
            });
            return Err(serde_json::to_string(&response).unwrap())
        }
    };

    let t = restaurant.get_table(order_request.table_id);

    let mut table = t.lock().unwrap();
    for item in &order_request.items {
        table.add_order(*item);
    }

    let response = json!({
        "success": true,
        "message": "Success!",
        "data": serde_json::to_string(&order_request).unwrap()
    });

    Ok(format!(
        "HTTP/1.1 200 OK\r\n\r\n{}",
        serde_json::to_string(&response).unwrap()
    ))
}

/// Handles a DELETE request for removing an order.
///
/// # Arguments
///
/// * `path`: A string containing the HTTP request path.
/// * `restaurant`: The restaurant instance.
///
/// # Returns
///
/// Returns a `Result` with either an HTTP response or an error message.
pub async fn handle_delete_order(
    path: &str,
    restaurant: Restaurant,
) -> Result<String, String> {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() == 4 {
        let table_id = parts[2].parse::<u32>().map_err(|_| "Invalid table id")?;
        let item_id = parts[3].parse::<u32>().map_err(|_| "Invalid item id")?;

        let t = restaurant.get_table(table_id);
        let result = t.lock().unwrap().remove_order(item_id);

        match result {
            Some(_) => {
                let response = json!({
                    "success": true,
                    "message": format!("Removed {} from table {}",
                        item_id, table_id
                    )
                });

                Ok(format!(
                    "HTTP/1.1 200 OK\r\n\r\n{}",
                    serde_json::to_string(&response).unwrap()
                ))
            },
            None => {
                let response = json!({
                    "success": false,
                    "message": "Order not found".to_string()
                });
                
                Err(serde_json::to_string(&response).unwrap())
            }
        }

    } else {
        let response = json!({
            "success": false,
            "message": "Invalid path".to_string()
        });
        
        Err(serde_json::to_string(&response).unwrap())
    }
}

/// Handles a GET request for retrieving order information.
///
/// # Arguments
///
/// * `path`: A string containing the HTTP request path.
/// * `restaurant`: The restaurant instance.
///
/// # Returns
///
/// Returns a `Result` with either an HTTP response or an error message.
pub async fn handle_get_order(path: &str, restaurant: Restaurant) -> Result<String, String> {
    let parts: Vec<&str> = path.split('/').collect();
    let table_id = parts[2].parse::<u32>().map_err(|_| "Invalid table id")?;
    let t = restaurant.get_table(table_id);
    let table = t.lock().unwrap();

    if parts.len() == 3 {   // `/orders/{table_id}`
        let orders = table.get_orders();

        let response = json!({
            "success": true,
            "message": "Success!",
            "data": serde_json::to_string(&orders).unwrap()
        });

        Ok(format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            serde_json::to_string(&response).unwrap()
        ))

    } else if parts.len() == 5 { // `/orders/{table_id}/items/{item_id}`
        let item_id = parts[4].parse::<u32>().map_err(|_| "Invalid item id")?;
        let order = table.get_order(item_id);

        let response = json!({
            "success": true,
            "message": "Success!",
            "data": serde_json::to_string(&order).unwrap()
        });

        Ok(format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            serde_json::to_string(&response).unwrap()
        ))

    } else {
        Err("Invalid path".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_restaurant(tables: usize, items: usize) -> Restaurant {
        let restaurant = Restaurant::new(tables);
        let table = restaurant.get_table(1);
        for i in 0..items {
            table.lock().unwrap().add_order(i as u32);
        }
        restaurant
    }

    #[tokio::test]
    async fn test_handle_post_order_ok() {
        // Create a sample request body
        let request = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": 2, \"items\": [101, 102]}";

        // Create a mock Restaurant
        let restaurant = init_restaurant(10, 5);

        // Call the function
        let result = handle_post_order(request, restaurant).await;

        // Check if the result is as expected
        assert!(result.is_ok());
        let response = result.unwrap();
        println!("test {:?}", response);
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("\\\"table_id\\\":2"));
        assert!(response.contains("\\\"items\\\":[101,102]"));
    }

    #[tokio::test]
    async fn test_handle_post_order_ng() {
        // String table id
        let request = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": st, \"items\": [101, 102]}";

        // Create a mock Restaurant
        let restaurant = init_restaurant(10, 5);
        let restaurant2 = restaurant.clone();

        // Call the function
        let result = handle_post_order(request, restaurant).await;

        // Check if the result is as expected
        assert!(result.is_err());
        let response = result.unwrap_err();
        assert!(response.contains("Failed to parse order request"));

        // String item id
        let request2 = "POST /orders HTTP/1.1\r\n\r\n{\"table_id\": 1, \"items\": [st, 102]}";
        
        // Call the function
        let result2 = handle_post_order(request2, restaurant2).await;
        // Check if the result is as expected
        assert!(result2.is_err());
        let response2 = result2.unwrap_err();
        assert!(response2.contains("Failed to parse order request"));
    }

    #[tokio::test]
    async fn test_handle_delete_order_ok() {
        // Create a sample path
        let path = "/orders/1/2"; // Assuming table_id = 1, item_id = 2

        // Create a mock Restaurant
        let restaurant = init_restaurant(10, 5);

        // Call the function
        let result = handle_delete_order(path, restaurant).await;

        // Check if the result is as expected
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("Removed 2 from table 1"));
    }

    #[tokio::test]
    async fn test_handle_get_all_orders_ok() {
        // Create a sample path
        let path = "/orders/1";

        // Create a mock Restaurant
        let restaurant = init_restaurant(10, 5);

        // Call the function
        let result = handle_get_order(path, restaurant).await;

        // Check if the result is as expected
        assert!(result.is_ok());
        let response = result.unwrap();

        // Assert that we get the expected response
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("\\\"item_id\\\":0,\\\"table_id\\\":1"));
        assert!(response.contains("\\\"item_id\\\":1,\\\"table_id\\\":1"));
        assert!(response.contains("\\\"item_id\\\":2,\\\"table_id\\\":1"));
        assert!(response.contains("\\\"item_id\\\":3,\\\"table_id\\\":1"));
        assert!(response.contains("\\\"item_id\\\":4,\\\"table_id\\\":1"));
        assert!(!response.contains("\\\"item_id\\\":5,\\\"table_id\\\":1"));
    }

    #[tokio::test]
    async fn test_handle_get_one_order_ok() {
        // Create a sample path
        let path = "/orders/1/items/3";

        // Create a mock Restaurant
        let restaurant = init_restaurant(10, 5);

        // Call the function
        let result = handle_get_order(path, restaurant).await;

        // Check if the result is as expected
        assert!(result.is_ok());
        let response = result.unwrap();

        // Assert that we get the expected response
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(!response.contains("\\\"item_id\\\":0,\\\"table_id\\\":1"));
        assert!(!response.contains("\\\"item_id\\\":1,\\\"table_id\\\":1"));
        assert!(!response.contains("\\\"item_id\\\":2,\\\"table_id\\\":1"));
        assert!(response.contains("\\\"item_id\\\":3,\\\"table_id\\\":1"));
        assert!(!response.contains("\\\"item_id\\\":4,\\\"table_id\\\":1"));
        assert!(!response.contains("\\\"item_id\\\":5,\\\"table_id\\\":1"));
    }
}
