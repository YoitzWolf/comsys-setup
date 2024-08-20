use clerk::csv_cl::clear_convert;

#[derive(Default, Debug, Clone)]
pub struct FileDetails {
    pub name: String,
    pub file_type: String,
    pub data: Vec<u8>,
}


#[derive(Default, Debug, Clone)]
pub struct ParsedTableFileDetails {
    pub name: String,
    pub headers: Vec<String>,
    pub value: Vec<Vec<String>>,
}

impl FileDetails {
    pub fn parse_to_table(&self, del: u8) -> Result<ParsedTableFileDetails, usize> {
        let res = clear_convert(self.data.as_slice(), del);

        match res {
            Ok((heads, dats)) => {
                Ok(ParsedTableFileDetails {
                    name: self.name.clone(),
                    headers: heads,
                    value: dats
                })
            }
            Err(x) => Err(x)
        }
    }
}

pub fn parse_to_table(name: String, data: &[u8], del: u8) -> Result<ParsedTableFileDetails, usize> {
    let res = clear_convert(data, del);

    match res {
        Ok((heads, dats)) => {
            Ok(ParsedTableFileDetails {
                name: name,
                headers: heads,
                value: dats
            })
        }
        Err(x) => Err(x)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileLoadStatus{
    Blocked,
    Waiting,
    Loading,
    Error,
    Finished(String)
}