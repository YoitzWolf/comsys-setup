use std::error::Error;
use std::fmt::Debug;
use std::io::Cursor;
use umya_spreadsheet::{Cell, Worksheet};
use umya_spreadsheet::helper::coordinate::string_from_column_index;
use super::traits::*;

// use crate::grpc::comp::{NominationDeclaration, Participant};

pub async fn merged_cleaner(wk: &mut Worksheet) -> Result<&mut Worksheet, Box<dyn Error>> {
    let cords = wk.get_merge_cells_mut().iter_mut().map(|x| {
        let x1 = x.get_coordinate_start_col().unwrap().get_num().clone();
        let y1 = x.get_coordinate_start_row().unwrap().get_num().clone();
        let x2 = x.get_coordinate_end_col().unwrap().get_num().clone();
        let y2 = x.get_coordinate_end_row().unwrap().get_num().clone();
        //// println!("{}", x.get_range());
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

/*enum TableLine<Item> {
    Item(Item),
    Marker(String),
    //Empty
}*/
#[derive(Debug, Clone)]
struct LocalPackage<Item>
where Item: Clone {
    pub marker: String,
    pub data: Vec<Item>
}

pub(crate) async fn read_column<Package, Item>(col: &Worksheet, l_poit: u32, size: u32) -> Result<Vec<Package>, Box<dyn Error>>
    where 
        Package: From<(String, Vec<Item>)> + Debug + Clone,
        Item: TryFrom<(i32, Vec<String>)> + Clone + Debug
{
    let mut cols = vec![];
    let mut L  = usize::MAX;
    let mut c;
    for i in l_poit..l_poit+size {
        c = col.get_collection_by_column(&i); 
        L = usize::min(L, c.len());
        cols.push(c);
    }
    let mut packages = vec![];
    let mut package = LocalPackage::<Item> {
        marker: "DEBUG".to_string(),
        data: vec![]
    };
    for line in 0..L {
        // println!(">>>");

        let ls = cols.iter().map(|x| x[line] ).collect::<Vec<&Cell>>();

        if let Some(f) = ls[0].get_value_number() {
            // Item
            package.data.push(
                //TableLine::Item::<Item>(
                match Item::try_from((f.round() as i32, ls.iter().skip(1).map(|x|{x.get_formatted_value().trim().to_string()}).collect())) {
                    Ok(v) => {v},
                    Err(e) => { {Err(Box::<dyn Error>::from(format!("Parce Error: line {}, {:?}", line+1, ls)))}? },
                }
                //)
            );
        } else {
            if let Some(non_empty) = ls.iter().map(|x| {x.get_formatted_value().trim().to_string()}).find(|x| {x.len() > 0}) {
                // Marker
                if package.data.len() > 0 { packages.push(package); }
                package = LocalPackage{
                    marker: non_empty.to_string(),
                    data: vec![]
                };
            } else {
                // Empty
            }
        }
    }
    if package.data.len() > 0 { packages.push(package); }

    // println!("{:?}", packages);

    Ok(packages.iter().map(|x| {
        Package::from((x.marker.clone(), x.data.clone())) 
    }).collect() )
}

pub async fn read_packaged_table<Package>(data: Vec<u8>, count_each: u32, linsize: u32, shift: u32, filter: impl Fn(&String) -> bool) 
    -> Result<
        Vec< Vec<Package> >,
        Box<dyn Error>
    > 
    where 
        Package: ConvertablePackage + Debug + Clone
{
    let mut resz = vec![];
    if let Ok(mut book) = umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(data), true){
        let collection = book.get_sheet_collection_mut();
        for sheet in collection.iter_mut() {
            let name = sheet.get_name();
            let cleaned = merged_cleaner(sheet).await?;
            
            for i in 0..count_each {
                let p = read_column::<Package, Package::Item>(
                    &cleaned,
                    shift + linsize*i,
                    linsize
                ).await?;
                resz.push(p);
            }
        };
    }    

    Ok(resz)
}


mod testing {
    use std::ops::Index;

    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    pub struct Team {
        /// team id ~ local id;
        pub tid: i32,
        /// Nomination
        //pub nom: String,
        /// Participants Ids
        pub participants: Vec<EasyParticipant>,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct NominationDeclaration {
        pub title: String,
        /// action id <-> team
        pub teams: std::collections::HashMap<i32, Team>,
        /// index = order, items = team ids
        /// generic.IntPair ages = 2;
        /// int64   group_size   = 3;
        pub inner_queue: Vec<i32>,
    }


    #[derive(Clone, PartialEq, Debug)]
    pub struct EasyParticipant {
        //pub uid: i32,
        pub name: String,
        pub extra_personal: Vec<String>,
    }
    
    impl From<( std::string::String, Vec<Team> )> for NominationDeclaration {
        fn from(value: ( std::string::String, Vec<Team> )) -> Self {
            Self{
                title: value.0,
                inner_queue: value.1.iter().map(|x| {x.tid.clone()}).collect(),
                teams: value.1.iter().map(|x| {(x.tid.clone(), x.clone())}).collect(),
            }
        }
    }

    impl TryFrom<(i32, Vec<String>)> for Team {
        type Error = Box<dyn Error>;
    
        fn try_from(value: (i32, Vec<String>)) -> Result<Self, Self::Error> {
            Ok (
                Team {
                    tid: value.0,
                    //nom: todo!(),
                    participants: value.1.index(0).split(',').map(
                        |x| {
                            EasyParticipant {
                                name: x.to_string(),
                                extra_personal: value.1.iter().skip(1).cloned().collect(),
                            }
                        }
                    ).collect(),
                }
            )
        }
    }

    impl ConvertablePackage for NominationDeclaration {
        type Item = Team;
    }

    #[tokio::test]
    async fn test_conv() {
        use std::fs;
        let data: Vec<u8> = fs::read(
            "C:\\Users\\yoitz\\Downloads\\стартовыйизмененный+программа_25_05_2024(1).xlsx"
        ).unwrap();
        // println!(
        //    "{:?}", data.len()
        //);
        let res = 
            read_packaged_table::<NominationDeclaration>(
                data, 2, 3, 1, |x: &String| {true}
            ).await;

        // println!(
        //    "{:?}", res
        //)

    }
}

