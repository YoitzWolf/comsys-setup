
pub use csv;

type CsvClRes = (Vec<String>, Vec<Vec<String>>);
pub fn clear_convert(data: &[u8], del: u8) -> Result<CsvClRes, usize> {

    let mut reader = csv::ReaderBuilder::new().delimiter(del).from_reader(data);
    let mut records: Vec<Vec<String>> = vec![];
    let mut last_record: &Vec<String> = &vec![];
    if !reader.has_headers() {
        return Err(0);
    }
    let heads: Vec<String> = reader.headers().unwrap().iter().map(|x| {x.to_string()}).collect();
    let size = heads.len();
    for (i, v) in reader.records().enumerate() {
        if let Ok(d) = v {
            let mut v = d.iter().collect::<Vec<&str>>();
            if v.len() != size {
                return Err(i);
            }
            records.push(
                v.iter_mut().enumerate().map(
                    |(i, v)| {
                        if v.len() == 0 {
                            last_record[i].clone()
                        } else {
                            v.to_string()
                        }
                    }).collect()
            );
            last_record = records.last().unwrap();
            if last_record.len() != size {
                return Err(i);
            }
        } else {
            return Err(i);
        }
    }

    Ok(
        (heads, records)
    )
}