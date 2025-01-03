use std::error::Error;
use std::io::Cursor;

use umya_spreadsheet::{Cell, Worksheet};

use crate::grpc::comp::{NominationDeclaration, Participant};
use super::conv;

enum TableLine {
    Item(i32, Vec::<String>),
    Marker(String),
    //Empty
}

pub fn read_column(col: &Worksheet, l_poit: u32, size: u32) -> Result<(), Box<dyn Error>> {
    let mut cols = vec![];
    let mut len  = usize::MAX;
    let mut c;
    for i in l_poit..l_poit+size {
        c = col.get_collection_by_column(&i); 
        len = usize::min(len, c.len());
        cols.push(c);
    }

    let mut nominations = vec![];

    let mut package = vec![];
    for line in 0..len {
        let ls = cols.iter().map(|x| x[line] ).collect::<Vec<&Cell>>();

        if let Some(f) = ls[0].get_value_number() {
            // Item
            package.push(TableLine::Item(f.round() as i32, ls.iter().skip(1).map(|x|{x.get_formatted_value().trim().to_string()}).collect()));
        } else {
            if let Some(non_empty) = ls.iter().map(|x| {x.get_formatted_value().trim().to_string()}).find(|x| {x.len() > 0}) {
                // Marker
                nominations.push(package);
                package = vec![];
                package.push(TableLine::Marker(non_empty.to_string()));
            } else {
                // Empty
            }
        }
    }   

    todo!()
}

pub async fn compet_reader(data: Vec<u8>) 
    -> Result<
        (Vec<Participant>, Vec<Vec<NominationDeclaration>>),
        Box<dyn Error>
    > 
{

    if let Ok(mut book) = umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(data), true){
        let collection = book.get_sheet_collection_mut();
        for sheet in collection.iter_mut() {
            let name = sheet.get_name();
            let cleaned = conv::merged_cleaner(sheet).await?;
            
        };
    }    

    todo!()
}




