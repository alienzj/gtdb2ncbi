extern crate calamine;
use calamine::{Xlsx, Reader};

use std::io::Cursor;

fn main() {
    println!("Hello, world!");
    const ARCHAEA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_archaea.xlsx");
    const BACTERIA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_bacteria.xlsx");

    let archaea_reader = Cursor::new(ARCHAEA_XLSX);
    let bacteria_reader = Cursor::new(BACTERIA_XLSX);

    let archaea_excel = Xlsx::new(archaea_reader).unwrap();
    let bacteria_excel = Xlsx::new(bacteria_reader).unwrap();

    assert_eq!(archaea_excel.sheet_names(), bacteria_excel.sheet_names());

}
