
use csv;
use csv::StringRecord;

fn main() {
    let data = "id;участники;субъект;;номинация\n\
                        1;Боба Бибович;Су ;Бъе;Расстрелы\n\
                        ;Бобина Бобикова;;;\n\
                        ;Бобстер Бонберович;;;\n\
                        2;Дементий Дементиевич;К;Т;Расстрелы2".to_string();
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_reader(data.as_bytes());
    let mut records: Vec<Vec<String>> = vec![];
    let mut last_record: &Vec<String> = &vec![];
    for i in reader.records() {
        if let Ok(d) = i {
            let mut v = d.iter().collect::<Vec<&str>>();
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
        }
    }
    println!("{:?}", records);
}