use std::error::Error;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::Index;
use std::path::Path;
use std::sync::Arc;
use umya_spreadsheet;
use umya_spreadsheet::{Coordinate, Worksheet};
use umya_spreadsheet::helper::coordinate::string_from_column_index;
use crate::grpc::comp::{Participant, Team};


pub type NaiveParsed = (Vec<std::string::String>, Vec<Vec<std::option::Option<std::string::String>>>);

pub async fn parce_naive(worksh: Arc<&Worksheet>) -> NaiveParsed {
    let mut headers: Vec<String> = vec![];
    let partic_sizes = worksh.get_highest_column_and_row();
    for part_col_i in 1..(partic_sizes.0+1) {
        headers.push(
            worksh.get_value((part_col_i, 1))
        );
    }
    let mut parsed = vec![];
    for part_row_i in 2..(partic_sizes.1+1) {
        let mut tup = vec![];
        for part_col_i in 1..(partic_sizes.0+1) {
            let cell = worksh.get_cell((part_col_i, part_row_i));
            if let Some(cell) = cell {
                tup.push(Some(cell.get_value().to_string()));
            } else {
                tup.push(None);
            }
        }
        parsed.push(tup);
    }
    (headers, parsed)
}

pub async fn parce_naive_struct<T>(worksh: Arc<&Worksheet>) -> Result<Vec<T>, T::Error>
    where T: TryFrom<(Vec<String>, Vec<String>)> + Debug
{
    let (hd, data) = parce_naive(worksh).await;
    //println!("Naive parser: {:?}", data);
    let mut res: Vec<Vec<String>> = vec![];
    for (i, x) in data.iter().enumerate() {
        let mut y = vec![];
        for j in 0..x.len() {
            y.push(
                x.index(j).clone().unwrap_or({
                    if res.is_empty() {
                        "".to_string()
                    } else {
                        res.index(i - 1).index(j).clone()
                    }
                })
            );
        };
        res.push(y);
    }
    drop(data);
    let mut result = vec![];
    for i in res {
        let t = T::try_from((hd.clone(), i.clone()))?;
        result.push(t); 
    }
    Ok(result)
}

pub async fn merged_cleaner(wk: &mut Worksheet) -> Result<&mut Worksheet, Box<dyn Error>> {
    let cords = wk.get_merge_cells_mut().iter_mut().map(|x| {
        let x1 = x.get_coordinate_start_col().unwrap().get_num().clone();
        let y1 = x.get_coordinate_start_row().unwrap().get_num().clone();
        let x2 = x.get_coordinate_end_col().unwrap().get_num().clone();
        let y2 = x.get_coordinate_end_row().unwrap().get_num().clone();
        //println!("{}", x.get_range());
        x.set_range(format!("{}{}:{}{}", string_from_column_index(&x1), y1, string_from_column_index(&x1), y1));
        (x1, y1, x2, y2)
    }).collect::<Vec<(u32,u32,u32,u32)>>();
    for (x1, y1, x2, y2) in cords {
        for i in x1..x2+1 {
            for j in y1..y2+1 {
                let val = wk.get_cell_value((x1, y1)).clone();
                wk.get_cell_mut((i, j)).set_cell_value( val );
            }
        }
    };
    Ok(wk)
}

pub async fn compet_reader(data: Vec<u8>) -> Result<(Vec<Participant>, Vec<Team>), Box<dyn Error>> {
    if let Ok(mut book) = umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(data), true){
        if let (Some(mut particips),Some(mut perfoms)) =
            (
                book.get_sheet_by_name_mut("Участники").cloned(),
                book.get_sheet_by_name_mut("Выступления").cloned()
            ) {

            let particips = merged_cleaner(&mut particips).await?;
            //println!("Particips cleaned");
            let perfoms = merged_cleaner(&mut perfoms).await?;
            //println!("Perfs cleaned");
            let parts = parce_naive_struct::<Participant>(Arc::new(particips)).await?;
            //println!("Particips struct parsed");
            let perfoms = parce_naive(Arc::new(perfoms)).await;
            //println!("Perfs naive parsed");
            let mut teams: Vec<Team> = vec![];
            for i in perfoms.1 {
                let tid = i.index(0).clone().unwrap().parse()?;
                let part_id = i.index(1).clone().unwrap().parse()?;
                let nom_id  = i.index(2).clone().unwrap();
                if !teams.is_empty() && teams.last().unwrap().tid == tid {
                    if let Some(x) = teams.last_mut() {
                        if !x.nom.eq(&nom_id) {
                            return Err("Invalid data: У одной группы выступления разные номинации".into());
                        }
                        x.participants.push(
                            part_id
                        );
                    } else {
                        return Err("Invalid data".into());
                    }

                } else {
                    teams.push(
                        Team {
                            tid,
                            nom: nom_id,
                            participants: vec![part_id],
                        }
                    )
                }

            }
            Ok((parts, teams))
        }else {
            Err("Неверная структура .xslx файла!".into())
        }
    } else {
        Err("Невозможно прочитать .xslx файл!".into())
    }
}

/*
pub mod test {
    use std::fs::read;
    use std::path::Path;
    use crate::xslx_conv::prelude::*;

    #[tokio::test]
    pub async fn test_umya() {
        let u = read(Path::new("C:\\Users\\yoitz\\Downloads\\Лист Microsoft Excel.xlsx")).unwrap();
        let res = compet_reader(u).await.unwrap();
        println!("{:?}", res);
    }
}*/