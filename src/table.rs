
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use super::order::Order;

pub struct Table {
    id: u32,
    orders: HashMap<u32, Order>
}

impl Table {
    pub fn new(table_id: u32) -> Table {
        Table {
            id: table_id,
            orders: HashMap::new()
        }
    }

    pub fn add_order(&mut self, item_id: u32) {
        let mut rng = thread_rng();
        let order = Order::new(item_id, self.id, rng.gen_range(5..16));
        self.orders.insert(item_id, order);
    }

    pub fn get_order(&self, item_id: u32) -> Option<&Order> {
        self.orders.get(&item_id)
    }

    pub fn get_orders(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }

    pub fn remove_order(&mut self, item_id: u32) -> Option<Order> {
        self.orders.remove(&item_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_order() {
        let mut table = Table::new(1);
        table.add_order(42);

        assert!(table.get_order(42).is_some());
    }

    #[test]
    fn test_get_order() {
        let mut table = Table::new(2);
        table.add_order(43);

        let order = table.get_order(43);
        assert!(order.is_some());
        assert_eq!(order.unwrap().item_id, 43);
    }

    #[test]
    fn test_get_orders() {
        let mut table = Table::new(3);
        table.add_order(44);
        table.add_order(45);

        let orders = table.get_orders();
        assert_eq!(orders.len(), 2);
    }

    #[test]
    fn test_remove_order() {
        let mut table = Table::new(4);
        table.add_order(46);

        let removed_order = table.remove_order(46);
        assert!(removed_order.is_some());
        assert_eq!(removed_order.unwrap().item_id, 46);
        assert!(table.get_order(46).is_none());
    }
}