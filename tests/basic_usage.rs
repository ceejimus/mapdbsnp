use std::io::{self, Read};
use std::{fs::File, path::PathBuf};

use mapdbsnp::index::create_map;
use mapdbsnp::mapper::map_tsv;
use mktemp::Temp;

fn cmp_files(path1: &str, path2: &str) -> io::Result<bool> {
    let mut f1 = File::open(path1)?;
    let mut f2 = File::open(path2)?;

    let mut b1 = Vec::new();
    let mut b2 = Vec::new();

    let _ = f1.read_to_end(&mut b1);
    let _ = f2.read_to_end(&mut b2);

    Ok(b2 == b1)
}

#[test]
fn test_make_map() {
    let dir = Temp::new_dir().unwrap();
    let dir_path = dir.to_path_buf();

    let out_map_path = dir_path.join("test.map");
    let expected_map_path = "tests/data/test.map.10.map";
    let src_tsv = "tests/data/test.map.10.tsv";

    create_map(&PathBuf::from(src_tsv), &out_map_path).unwrap();
    cmp_files(expected_map_path, out_map_path.to_str().unwrap()).unwrap();
}

#[test]
fn test_map_markers() {
    let dir = Temp::new_dir().unwrap();
    let dir_path = dir.to_path_buf();

    let out_tsv_path = dir_path.join("out.tsv");
    let expected_tsv_path = "tests/data/test.out.10.tsv";
    let src_tsv_path = "tests/data/test.in.10.tsv";
    let src_map_path = "tests/data/test.map.10.map";

    map_tsv(
        &PathBuf::from(src_tsv_path),
        &out_tsv_path,
        &PathBuf::from(src_map_path),
    )
    .unwrap();
    cmp_files(expected_tsv_path, out_tsv_path.to_str().unwrap()).unwrap();
}
