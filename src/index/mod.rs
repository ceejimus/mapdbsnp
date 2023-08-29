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

// TODO: a side-effect of this pattern is that the map is partially created
// we can handle these errors and delete the result if we want
pub fn create_map<R: std::io::Read, P: AsRef<Path>>(rdr: R, dst: &P) -> anyhow::Result<()> {
    // Read TSV records into iter and capture first error
    // TODO: encapsulate this in something
    let csv_records = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(rdr)
        .into_records();

    let mut csv_error = Result::Ok(());
    let records_iter = csv_records.scan(&mut csv_error, |err, record| match record {
        Ok(record) => Some(record),
        Err(e) => {
            **err = Err(e);
            None
        }
    });
    // Parse TSV StringRecord iter into MapRecord iter
    // TODO: encapsulate this in something
    let mut parse_error = Result::Ok(());
    let map_records =
        records_iter
            .map(parse_tsv_map_record)
            .scan(&mut parse_error, |err, map_record| match map_record {
                Ok(map_record) => Some(map_record),
                Err(e) => {
                    **err = Err(e);
                    None
                }
            });
    // Write map records to dst
    let num_records = write_map_records(dst, map_records)?;
    // Return if read errors
    csv_error?;
    // Return if parse errors
    parse_error?;
    // Prepend file with number of records
    // Make this an append and change how suffix is read
    prepend_file(&num_records.to_be_bytes(), dst)?;

    Ok(())
}

fn parse_tsv_map_record(r: StringRecord) -> anyhow::Result<MapRecord> {
    let mut parts = r[1].split(':');
    let (chrom, pos) = match (parts.next(), parts.next()) {
        (Some(chrom), Some(pos)) => Ok((chrom, pos)),
        (_, _) => Err(anyhow!("Invalid TSV record")),
    }?;
    MapRecord::new(&r[0], chrom, pos)
}

// TODO: move this to binfmt
fn write_map_records<P: AsRef<Path>>(
    dst: &P,
    map_records: impl Iterator<Item = MapRecord>,
) -> anyhow::Result<u64> {
    // scope of mapfile
    // we want to make sure mapfile is flushed and dropped before we prepend num_records
    let mut map_wtr = BufWriter::new(File::create(dst)?);

    // runtime check if file is sorted and panic if not
    let mut last_rsid = 0;

    let mut num_records: u64 = 0;

    for record in map_records {
        write_map_record(&mut map_wtr, &record)?;
        num_records += 1;

        if last_rsid > record.rsid {
            panic!("Make sure source map is sorted.")
        }

        last_rsid = record.rsid;
    }
    map_wtr.flush()?;

    Ok(num_records)
}
