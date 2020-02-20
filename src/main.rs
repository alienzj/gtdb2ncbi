extern crate calamine;
use calamine::{Xlsx, Reader};

use std::io::Cursor;
use std::collections::HashMap;

fn main() {
    const ARCHAEA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_archaea.xlsx");
    const BACTERIA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_bacteria.xlsx");

    let archaea_map = parse(ARCHAEA_XLSX);
    let bacteria_map = parse(BACTERIA_XLSX);

    println!("{:?}", archaea_map);
    println!("{:?}", bacteria_map);
}

fn parse(raw_xlsx: &[u8]) -> HashMap<String, Vec<String>> {
    let reader = Cursor::new(raw_xlsx);

    let mut excel: Xlsx<_> = Xlsx::new(reader).unwrap();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for s in excel.sheet_names().to_owned() {
        if let Some(Ok(range)) = excel.worksheet_range(&s) {
            for r in range.rows().skip(1) {
                if let Some(gtdb) = r[3].get_string() {
                    for lineage in gtdb.split(',') {
                        if lineage.contains('(') {
                            let lineages: Vec<&str> = lineage.trim().split(|c| c == '(' || c == ')').collect();
                            if let Some(ncbi) = r[0].get_string() {
                                // println!("{}\t{}\t{}\n", ncbi, lineages[0], lineages[1]);
                                map.insert(String::from(lineages[0]), vec!(String::from(ncbi), String::from(lineages[1])));
                            }
                        } else {
                            let lineages: Vec<&str> = lineage.trim().rsplitn(2, ' ').collect();
                            if let Some(ncbi) = r[0].get_string() {
                                // println!("{}\t{}\n", ncbi, lineages[1]);
                                map.insert(String::from(lineages[1]), vec!(String::from(ncbi)));
                            }
                        }
                    }
                }
            }
        }
    }
    map
}