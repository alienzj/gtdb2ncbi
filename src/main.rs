extern crate calamine;
use calamine::{Xlsx, Reader};

use std::io::Cursor;
use std::collections::HashMap;

fn main() {
    const ARCHAEA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_archaea.xlsx");
    const BACTERIA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_bacteria.xlsx");

    let archaea_reader = Cursor::new(ARCHAEA_XLSX);
    let bacteria_reader = Cursor::new(BACTERIA_XLSX);

    let mut archaea_excel: Xlsx<_> = Xlsx::new(archaea_reader).unwrap();
    let mut bacteria_excel: Xlsx<_> = Xlsx::new(bacteria_reader).unwrap();

    let mut archaea_map: HashMap<String, Vec<String>> = HashMap::new();

    for s in archaea_excel.sheet_names().to_owned() {
        if let Some(Ok(range)) = archaea_excel.worksheet_range(&s) {
            for r in range.rows().skip(1) {
                if let Some(tax) = r[3].get_string() {
                    for lineage in tax.split(',') {
                        if lineage.contains('(') {
                            let lineages: Vec<&str> = lineage.trim().split(|c| c == '(' || c == ')').collect();
                            println!("{}\t{}\t{}\n", r[0], lineages[0], lineages[1]);
                        } else {
                            let lineages: Vec<&str> = lineage.trim().rsplitn(2, ' ').collect();
                            println!("{}\t{}\n", r[0], lineages[1]);
                        }
                    }
                }
            }
        }
    }

    assert_eq!(archaea_excel.sheet_names(), bacteria_excel.sheet_names());

}
