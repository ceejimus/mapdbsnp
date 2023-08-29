use crate::binfmt::MapRecord;
use crate::binfmt::{prepend_file, write_map_record};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::anyhow;
use csv::{Reader, ReaderBuilder, StringRecord};

struct TsvMapRecordReader<R> {
    csv_reader: Reader<R>,
}

impl<R: std::io::Read> TsvMapRecordReader<R> {
    fn new(rdr: R) -> anyhow::Result<Self> {
        let csv_reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(rdr);

        Ok(Self { csv_reader })
    }
}

impl<R: std::io::Read> Iterator for TsvMapRecordReader<R> {
    type Item = anyhow::Result<MapRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_record = StringRecord::new();
        match self.csv_reader.read_record(&mut next_record) {
            Ok(more_records) => {
                if !more_records {
                    return None;
                }

                match parse_tsv_map_record(next_record) {
                    Ok(map_record) => Some(Ok(map_record)),
                    Err(e) => Some(Err(anyhow!(e))),
                }
            }
            Err(e) => Some(Err(anyhow!(e))),
        }
    }
}

fn parse_tsv_map_record(r: StringRecord) -> anyhow::Result<MapRecord> {
    let mut parts = r[1].split(':');
    MapRecord::new(&r[0], parts.next().unwrap(), parts.next().unwrap())
}

pub fn create_map<R: std::io::Read, P: AsRef<Path>>(src_tsv: R, dst: &P) -> anyhow::Result<()> {
    let map_records_reader = TsvMapRecordReader::new(src_tsv)?;

    let num_records = write_map_records(dst, map_records_reader)?;
    prepend_file(&num_records.to_be_bytes(), dst)?;

    Ok(())
}

fn write_map_records<P: AsRef<Path>>(
    dst: &P,
    map_records: impl Iterator<Item = anyhow::Result<MapRecord>>,
) -> anyhow::Result<u64> {
    // scope of mapfile
    // we want to make sure mapfile is flushed and dropped before we prepend num_records
    let mut map_wtr = BufWriter::new(File::create(dst)?);

    // runtime check if file is sorted and panic if not
    let mut last_rsid = 0;

    let mut num_records: u64 = 0;

    for record in map_records {
        // let r = r?;
        // let record = parse_tsv_map_record(r)?;
        match record {
            Ok(record) => {
                write_map_record(&mut map_wtr, &record)?;
                num_records += 1;

                if last_rsid > record.rsid {
                    panic!("Make sure source map is sorted.")
                }

                last_rsid = record.rsid;
            }
            Err(e) => return Err(anyhow!(e)),
        }
    }
    map_wtr.flush()?;

    Ok(num_records)
}
