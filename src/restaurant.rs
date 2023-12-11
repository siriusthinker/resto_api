use std::sync::{Arc, Mutex};

use super::table::Table;

type TablePtr = Arc<Mutex<Table>>;

#[derive(Clone)]
pub struct Restaurant {
    tables: Vec<TablePtr>,
}

impl Restaurant {
    pub fn new(number_of_tables: usize) -> Restaurant {
        let mut tables = Vec::new();

        tables.reserve(number_of_tables);

        for tid in 0..number_of_tables as u32 {
            tables.push(Arc::new(Mutex::new(Table::new(tid))));
        }

        Restaurant { tables: tables }
    }

    pub fn get_table(&self, table_id: u32) -> TablePtr {
        Arc::clone(&self.tables[table_id as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_restaurant() {
        let num_tables = 5;
        let restaurant = Restaurant::new(num_tables);
        let actual_num_tables = restaurant.tables.len();

        assert_eq!(actual_num_tables, num_tables);
    }

    #[test]
    fn test_get_table() {
        let num_tables = 3;
        let restaurant = Restaurant::new(num_tables);

        let table_id = 1;
        let table_ptr = restaurant.get_table(table_id);

        assert!(table_ptr.lock().is_ok()); // Check if the mutex can be locked
    }
}