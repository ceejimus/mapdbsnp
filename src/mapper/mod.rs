use std::path::Path;

use anyhow::anyhow;
use csv::{StringRecord, StringRecordsIter};

use crate::binfmt::{
    maprecord::{rsid_to_u32, u8_to_chrom},
    MapReader,
};

pub fn map_tsv<P: AsRef<Path>>(src_tsv: P, dst_tsv: P, map: P) -> anyhow::Result<()> {
    // make TSV reader
    let mut src_tsv = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(src_tsv)?;
    // make TSV writer
    let mut dst_tsv = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(dst_tsv)?;
    // make MapReader
    let mut map_rdr = MapReader::new(map)?;
    // write mapped records
    for mapped_record in map_to_loci(&mut src_tsv.records(), &mut map_rdr)? {
        match mapped_record {
            Ok(r) => dst_tsv.write_record(&r)?,
            Err(e) => return Err(anyhow!(e)),
        };
    }
    Ok(())
}

fn map_to_loci<'r, R: std::io::Read>(
    src_records: &'r mut StringRecordsIter<'r, R>,
    map_rdr: &'r mut MapReader,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<StringRecord>> + 'r> {
    Ok(src_records.map(|r| match r {
        Ok(src_record) => map_record(src_record, map_rdr),
        Err(e) => Err(anyhow!(e)),
    }))
}

fn map_record(r: StringRecord, map_rdr: &mut MapReader) -> anyhow::Result<StringRecord> {
    // get lookup key from record
    let mut record_iter = r.iter();
    let rsid = rsid_to_u32(record_iter.next().ok_or(anyhow!("Invalid TSV record"))?)?;
    // use map reader to find record
    match map_rdr.find_record(rsid)? {
        Some(r) => {
            let chrom = u8_to_chrom(r.chrom)?;
            let pos = r.pos;
            let loci = format!("{}:{}", chrom, pos);
            let mut new_record = StringRecord::new();
            new_record.push_field(&loci);
            for field in record_iter {
                new_record.push_field(field);
            }
            Ok(new_record)
        }
        None => Err(anyhow!("Record not found in map: <key={}>", rsid)),
    }
}
