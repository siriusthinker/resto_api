use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Order {
    pub item_id: u32,
    pub table_id: u32,
    pub waiting_time: u32,
}

impl Order {
    pub fn new(item_id: u32, table_id: u32, waiting_time: u32) -> Order {
        Order {
            item_id: item_id,
            table_id: table_id,
            waiting_time: waiting_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() -> Result<(), String> {
        let order = Order::new(10, 2, 5);

        assert_eq!(
            order,
            Order {
                item_id: 10,
                table_id: 2,
                waiting_time: 5,
            }
        );
        Ok(())
    }
}