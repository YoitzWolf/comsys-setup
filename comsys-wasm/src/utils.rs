use chrono::prelude::*;
use std::time::SystemTime;
use prost_wkt_types::Timestamp;

pub fn get_viewport_width() -> i32 {
    web_sys::window()
        .expect("There should be a window")
        .inner_width()
        .expect("The window should have Some width")
        .as_f64()
        .expect("The width should be a number") as i32
}

pub fn parse_dateinput(input: String) -> Option<Timestamp> {
    web_sys::console::log_1(&format!("Dt:{}", input).to_string().into());
    //let sp = input.splitn(3, "-").collect::<Vec<&str>>();
    if let Ok(naive) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        let syst: SystemTime = Local.from_local_datetime(&naive.into()).unwrap().into();
        Some(syst.into())
    } else {
        None
    }
}