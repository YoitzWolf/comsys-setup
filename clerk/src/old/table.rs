use crate::record::{KeyMapper, Record};

pub trait Table<TableRecord: Record> {
    fn push(&mut self, rec: TableRecord);
    fn index(&self, index: usize) -> Option<&TableRecord>;
    fn index_set(&mut self, index: usize, val: TableRecord) -> Option<&TableRecord >;

    fn generate_text_view(&self) -> Vec<Vec<String>>;

    fn generate_header(&self) -> Vec<String> {
        TableRecord::KeyMap::keys()
    }
}