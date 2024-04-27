


pub fn get_viewport_width() -> i32 {
    web_sys::window()
        .expect("There should be a window")
        .inner_width()
        .expect("The window should have Some width")
        .as_f64()
        .expect("The width should be a number") as i32
}