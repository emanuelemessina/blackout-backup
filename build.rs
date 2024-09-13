extern crate winres;
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("res/blackout.ico", "icon");
    res.compile().unwrap();
}