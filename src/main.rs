extern crate calamine;
use calamine::{Xlsx, Reader};

use std::io::Cursor;
use std::collections::HashMap;
use std::result::Result;

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

fn parse2(raw_xlsx: &[u8]) -> Result<HashMap<String, Vec<String>>, (String, String)> {
    let reader = Cursor::new(raw_xlsx);
    let mut excel: Xlsx<_> = Xlsx::new(reader).unwrap();
    let sheets = excel.sheet_names().to_owned();
    
    let mut tax_map: HashMap<String, Vec<String>> = HashMap::new();

    sheets
        .iter()
        .map(|name| {
            let range = excel.worksheet_range(name);
            match range {
                None => Err((name.clone(), format!("sheet {} is empty", name))),
                Some(Err(err)) => Err((name.clone(), format!("{}", err))),
                Some(Ok(sheet)) => Ok({
                    sheet
                    .rows()
                    .skip(1)
                    .enumerate()
                    .map(|(i, row)| 
                        match row[0].get_string() {
                            None => Err((name.clone(), format!("sheet {} row:{} col:1 is empty", name, i))),
                            Some(ncbi) => {
                                match row[3].get_string() {
                                    None => Err((name.clone(), format!("sheet {} row:{} col:4 is empty", name, i))),
                                    Some(gtdb) => Ok({
                                        gtdb
                                            .split(',')
                                            .map(|lineage|
                                                if lineage.contains('(') {
                                                    let lineages: Vec<&str> = lineage.trim().split(|c| c == '(' || c == ')').collect();
                                                    tax_map.insert(String::from(lineages[0]), vec!(String::from(ncbi), String::from(lineages[1])));
                                                } else {
                                                    let lineages: Vec<&str> = lineage.trim().rsplitn(2, ' ').collect();
                                                    tax_map.insert(String::from(lineages[1]), vec!(String::from(ncbi)));
                                                }
                                            )
                                    })
                                }
                            }
                        }
                    )
                })
            }
        });

    Ok(tax_map)
}