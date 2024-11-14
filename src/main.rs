use calamine::{Reader, Xlsx};
use clap::{App, Arg};
use csv::{ReaderBuilder, Trim, WriterBuilder};

use std::collections::HashMap;
use std::io::Cursor;
use std::process;

fn main() {
    const ARCHAEA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_r220_archaea.xlsx");
    const BACTERIA_XLSX: &[u8] = include_bytes!("../data/ncbi_vs_gtdb_r220_bacteria.xlsx");

    let archaea_map = parse(ARCHAEA_XLSX);
    let bacteria_map = parse(BACTERIA_XLSX);

    let matches = App::new("gtdb2ncbi")
        .version("0.1")
        .author("alienzj <alienchuj@gmail.com>")
        .about("convert taxonomy system from GTDB to NCBI")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("GTDB taxonomy input file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("NCBI taxonomy output file")
                .takes_value(true),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap_or_else(|| {
        eprintln!("please supply --input [FILE]");
        process::exit(1);
    });
    let output = matches.value_of("output").unwrap_or_else(|| {
        eprintln!("please supply --output [FILE]");
        process::exit(1);
    });

    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .delimiter(b'\t')
        .from_path(input)
        .unwrap();

    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output)
        .unwrap();

    let mut headers = rdr.headers().unwrap().to_owned();
    headers.push_field("NCBI_taxonomy");

    wtr.write_record(&headers).unwrap();

    for result in rdr.records() {
        let mut record = result.unwrap();
        // we assume the gtdb classification locate at the 3th column
        let classification = record.get(2).unwrap();

        let lineages: Vec<&str> = classification.split(';').rev().collect();

        let mut classification_ncbi = String::new();
        let mut classification_sensitive = String::new();

        for i in 0..(lineages.len() - 1) {
            match archaea_map.get(lineages[i]) {
                Some(v) => {
                    if v.len() == 2 {
                        classification_ncbi = String::from(";") + &v[1] + &classification_ncbi;
                        classification_sensitive = String::from(";")
                            + &v[1]
                            + &String::from("__NCBI")
                            + &classification_sensitive;
                    } else {
                        if lineages[i + 1] == v[2] {
                            classification_ncbi = String::from(";") + &v[1] + &classification_ncbi;
                            classification_sensitive = String::from(";")
                                + &v[1]
                                + &String::from("__NCBI")
                                + &classification_sensitive;
                        } else {
                            classification_ncbi = String::from(";") + &v[1] + &classification_ncbi;
                            classification_sensitive = String::from(";")
                                + &v[1]
                                + &String::from("__NCBI?")
                                + &classification_sensitive;
                        }
                    }
                }
                None => match bacteria_map.get(lineages[i]) {
                    Some(v) => {
                        if v.len() == 2 {
                            classification_ncbi = String::from(";") + &v[1] + &classification_ncbi;
                            classification_sensitive = String::from(";")
                                + &v[1]
                                + &String::from("__NCBI")
                                + &classification_sensitive;
                        } else {
                            if lineages[i + 1] == v[2] {
                                classification_ncbi =
                                    String::from(";") + &v[1] + &classification_ncbi;
                                classification_sensitive = String::from(";")
                                    + &v[1]
                                    + &String::from("__NCBI")
                                    + &classification_sensitive;
                            } else {
                                classification_ncbi =
                                    String::from(";") + &v[1] + &classification_ncbi;
                                classification_sensitive = String::from(";")
                                    + &v[1]
                                    + &String::from("__NCBI?")
                                    + &classification_sensitive;
                            }
                        }
                    }
                    None => {
                        let level: Vec<&str> = lineages[i].split("__").collect();
                        if level[1] == "" {
                            classification_ncbi =
                                String::from(";") + &lineages[i] + &classification_ncbi;
                            classification_sensitive =
                                String::from(";") + &lineages[i] + &classification_sensitive;
                        } else {
                            classification_ncbi = String::from(";")
                                + &level[0]
                                + &String::from("__NCBI_UNKNOWN")
                                + &classification_ncbi;
                            classification_sensitive = String::from(";")
                                + &level[0]
                                + &String::from("__NCBI_UNKNOWN")
                                + &classification_sensitive;
                        }
                    }
                },
            }
        }

        classification_ncbi = String::from("") + lineages.last().unwrap() + &classification_ncbi;
        classification_sensitive =
            String::from("") + lineages.last().unwrap() + &classification_sensitive;

        println!("{}", classification);
        println!("{}", classification_sensitive);
        println!("{}\n", classification_ncbi);

        record.push_field(&classification_ncbi);

        wtr.write_record(&record).unwrap();
    }

    wtr.flush().unwrap();
}

fn parse(raw_xlsx: &[u8]) -> HashMap<String, Vec<String>> {
    let reader = Cursor::new(raw_xlsx);

    let mut excel: Xlsx<_> = Xlsx::new(reader).unwrap();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for s in excel.sheet_names().to_owned() {
        if let Some(Ok(range)) = excel.worksheet_range(&s) {
            for r in range.rows().skip(1) {
                if let Some(gtdb) = r[3].get_string() {
                    if let Some(ncbi) = r[0].get_string() {
                        for lineage in gtdb.split(',') {
                            if lineage.contains('(') {
                                let lineages: Vec<&str> = lineage
                                    .trim()
                                    .split(|c| c == '(' || c == ')' || c == '%')
                                    .collect();

                                let key = String::from(lineages[0]);
                                let percent = lineages[2].trim().trim_start_matches('<');

                                if !map.contains_key(&key) {
                                    map.insert(
                                        key,
                                        vec![
                                            String::from(percent),
                                            String::from(ncbi),
                                            String::from(lineages[1]),
                                        ],
                                    );
                                } else {
                                    println!("percent: {}", percent);
                                    let percent_1: f32 = map.get(&key).unwrap()[0].parse().unwrap();
                                    let percent_2: f32 = percent.parse().unwrap();
                                    if (percent_2 - percent_1) > 20.0 {
                                        map.insert(
                                            key,
                                            vec![
                                                String::from(percent),
                                                String::from(ncbi),
                                                String::from(lineages[1]),
                                            ],
                                        );
                                    }
                                }
                            } else {
                                let lineages: Vec<&str> = lineage.trim().rsplitn(2, ' ').collect();

                                let key = String::from(lineages[1]);
                                let percent = lineages[0]
                                    .trim()
                                    .trim_start_matches('<')
                                    .trim_end_matches('%');

                                if !map.contains_key(&key) {
                                    map.insert(
                                        key,
                                        vec![String::from(percent), String::from(ncbi)],
                                    );
                                } else {
                                    println!("percent: {}", percent);
                                    let percent_1: f32 = map.get(&key).unwrap()[0].parse().unwrap();
                                    let percent_2: f32 = percent.parse().unwrap();
                                    if (percent_2 - percent_1) > 20.0 {
                                        map.insert(
                                            key,
                                            vec![String::from(percent), String::from(ncbi)],
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/*
fn parse_gtdb(gtdb: &str, ncbi: &str) -> Vec<(String, Vec<String>)> {
    gtdb.split(',')
        .map(|lineage| {
            if lineage.contains('(') {
                let lineages: Vec<&str> = lineage.trim().split(|c| c == '(' || c == ')').collect();
                (
                    String::from(lineages[0]),
                    vec![String::from(ncbi), String::from(lineages[1])],
                )
            } else {
                let lineages: Vec<&str> = lineage.trim().rsplitn(2, ' ').collect();
                (String::from(lineages[1]), vec![String::from(ncbi)])
            }
        })
        .collect::<Vec<(String, Vec<String>)>>()
}

fn parse_sheet(
    sheet: Range<DataType>,
    name: &str,
) -> Result<Vec<Result<Vec<(String, Vec<String>)>, String>>, String> {
    Ok(sheet
        .rows()
        .skip(1)
        .map(|row| match row[0].get_string() {
            None => Err(format!("sheet {} is empty", name)),
            Some(ncbi) => match row[3].get_string() {
                None => Err(format!("sheet{} is empty", name)),
                Some(gtdb) => Ok(parse_gtdb(gtdb, ncbi)),
            },
        })
        .collect())
}

fn parse_excel(raw_xlsx: &[u8]) -> Result<HashMap<String, Vec<String>>, String> {
    let reader = Cursor::new(raw_xlsx);
    let mut excel: Xlsx<_> = Xlsx::new(reader).unwrap();

    let mut tax_map: HashMap<String, Vec<String>> = excel
        .sheet_names()
        .into_iter()
        .map(|name| match excel.worksheet_range(name) {
            None => Err(format!("sheet {} is empty", name)),
            Some(Err(err)) => Err(format!("{}", err)),
            Some(Ok(sheet)) => parse_sheet(sheet, name),
        })
        // FIXME
        .collect();

    Ok(tax_map)
}
*/

#[test]
fn split() {
    assert_eq!("s__".trim().split("__").collect::<Vec<&str>>()[1], "");
}

#[test]
fn split_() {
    assert_eq!("s__a".trim().split("__").collect::<Vec<&str>>().len(), 1);
}
