use crate::binfmt::{MapRecord, MapWriter};
use std::{fs::File, path::Path};

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
pub fn create_map<P: AsRef<Path>>(src_tsv_path: &P, mapfile_path: &P) -> anyhow::Result<()> {
    // Make TSV reader
    let mut tsv_reader = TsvMapRecordReader::new(File::open(src_tsv_path)?);
    // Make MapWriter
    let mut map_wtr = MapWriter::new(mapfile_path)?;
    // Write map records to dst
    map_wtr.write_map(&mut tsv_reader)?;
    // Check TSV read errors
    tsv_reader.ok()?;

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
