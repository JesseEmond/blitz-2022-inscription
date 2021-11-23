// Precompute rectangles of a given area, then store to disk.
// Used by harder instances of the problem to treat the task as "rectangle packing".

extern crate application;

use application::{
    rect_inventory::RectangleInventory,
};
use clap::{Arg, App};
use std::path::Path;

fn is_valid_area(area: String) -> Result<(), String> {
    match area.parse::<usize>() {
        Ok(area) => area,
        Err(_) => {
            return Err(String::from("area must be a positive integer"));
        }
    };
    Ok(())
}


fn main() {
    let matches = App::new("Coveo 2022 Inscription Rectangle Precomputation")
                          .arg(Arg::with_name("max_area")
                               .value_name("MAX_AREA")
                               .long("area")
                               .help("Precompute rectangles up to this area")
                               .required(true)
                               .validator(is_valid_area))
                          .get_matches();
    let area = matches.value_of("max_area").unwrap();
    let area: usize = area.parse().unwrap();

    let inventory = RectangleInventory::from_scratch(/*max_area=*/area);
    let filename = format!("src/precomputed_area_{}.rects", area);
    inventory.save(Path::new(&filename)).expect("Failed to save precomputed rectangles.");
}