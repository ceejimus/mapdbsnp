use std::io::{self, Cursor};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub(crate) const RECORD_SIZE: u64 = 4 + 1 + 4;

pub(crate) struct MapRecord {
    pub rsid: u32,
    pub chrom: u8,
    pub pos: u32,
}

impl MapRecord {
    pub(crate) fn new(rsid: &str, chrom: &str, pos: &str) -> anyhow::Result<Self> {
        let rsid = rsid_to_u32(rsid)?;
        let chrom = chrom_to_u8(chrom)?;
        let pos = pos.parse::<u32>()?;
        Ok(Self { rsid, chrom, pos })
    }

    pub(crate) fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.write_u32::<BigEndian>(self.rsid)?;
        bytes.write_u8(self.chrom)?;
        bytes.write_u32::<BigEndian>(self.pos)?;

        Ok(bytes)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> io::Result<MapRecord> {
        let mut cursor = Cursor::new(bytes);
        let rsid = cursor.read_u32::<BigEndian>()?;
        let chrom = cursor.read_u8()?;
        let pos = cursor.read_u32::<BigEndian>()?;

        Ok(MapRecord { rsid, chrom, pos })
    }
}

pub(crate) fn rsid_to_u32(rsid: &str) -> anyhow::Result<u32> {
    Ok(rsid.replace("rs", "").parse::<u32>()?)
}

fn chrom_to_u8(chrom: &str) -> anyhow::Result<u8> {
    match chrom {
        "X" => Ok(23),
        "Y" => Ok(24),
        "MT" => Ok(25),
        _ => Ok(chrom.parse::<u8>()?),
    }
}

pub(crate) fn u8_to_chrom(x: u8) -> anyhow::Result<String> {
    Ok(match x {
        1..=22 => format!("{x}"),
        23 => "X".into(),
        24 => "Y".into(),
        25 => "MT".into(),
        _ => panic!("Invalid chrom representation {}", x),
    })
}
