use std::collections::HashMap;


#[derive(Debug, Default, Clone)]
pub struct GenericValue {
    pub values: Vec<String>
}

#[derive(Debug, Default, Clone)]
pub struct GenericTable {
    columns: HashMap<String, Vec<GenericValue>>,
    size: usize,
}


impl GenericTable {

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn remove_col(&mut self, val: &str) -> Result<Vec<GenericValue>, ()> {
        if let Some(v) = self.columns.remove(val) {
            Ok(v)
        } else {
            Err(())
        }
    }

    pub fn add_col(&mut self, name: String) -> Result<(), ()> {
        if self.columns.contains_key(&name) { return Err(()) }
        else {
            self.columns.insert(name, vec![GenericValue::default(); self.size()]);
            Ok(())
        }
    }
}