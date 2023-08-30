use crate::binfmt::MapRecord;
use crate::binfmt::{prepend_file, write_map_record};
use std::{fs::File, io::BufWriter, path::Path};

use anyhow::anyhow;
use csv::{ReaderBuilder, StringRecord, StringRecordsIntoIter};

struct TsvMapRecordReader<R> {
    inner: StringRecordsIntoIter<R>,
    error: Option<anyhow::Error>,
}

impl<R: std::io::Read> TsvMapRecordReader<R> {
    fn new(rdr: R) -> Self {
        let inner = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(rdr)
            .into_records();

        Self { inner, error: None }
    }

    fn ok(self) -> anyhow::Result<()> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl<R: std::io::Read> Iterator for TsvMapRecordReader<R> {
    type Item = MapRecord;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: can use map?
        match self.inner.next() {
            Some(r) => match r {
                Ok(r) => match parse_tsv_map_record(r) {
                    Ok(mt) => Some(mt),
                    Err(e) => {
                        self.error = Some(anyhow!(e));
                        None
                    }
                },
                Err(e) => {
                    self.error = Some(anyhow!(e));
                    None
                }
            },
            None => None,
        }
    }
}

// TODO: a side-effect of this pattern is that the map is partially created
// we can handle these errors and delete the result if we want
pub fn create_map<P: AsRef<Path>>(rdr: impl std::io::Read, dst: &P) -> anyhow::Result<()> {
    // Make TSV reader
    let mut tsv_reader = TsvMapRecordReader::new(rdr);
    // Make Map writer
    let mut map_wtr = BufWriter::new(File::create(dst)?);
    // Write map records to dst
    let num_records = write_map_records(&mut map_wtr, &mut tsv_reader)?;
    // Check TSV read errors
    tsv_reader.ok()?;
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
fn write_map_records(
    wtr: &mut impl std::io::Write,
    map_records: impl Iterator<Item = MapRecord>,
) -> anyhow::Result<u64> {
    // scope of mapfile
    // we want to make sure mapfile is flushed and dropped before we prepend num_records

    // runtime check if file is sorted and panic if not
    let mut last_rsid = 0;

    let mut num_records: u64 = 0;

    for record in map_records {
        write_map_record(wtr, &record)?;
        num_records += 1;

        if last_rsid > record.rsid {
            panic!("Make sure source map is sorted.")
        }

        last_rsid = record.rsid;
    }
    wtr.flush()?;

    Ok(num_records)
}
