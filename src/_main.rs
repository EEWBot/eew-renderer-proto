pub mod quake_prefecture {
    include!(concat!(env!("OUT_DIR"), "/quake_prefecture.rs"));
}
use prost::Message;
use quake_prefecture::{CodeArray, Epicenter};

use base65536::{WrapOptions, encode};

use std::fs::File;
use std::io::Write;
use std::vec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rendering_width: u32 = 600;
    let quaked_unixtime: u64 = 1719492256;
    let epi = Epicenter {
        lat_x10: 35,
        lon_x10: 135,
    };
    let magnitude_x10: u32 = 100;
    let code_table_difinition: &str = "hoge";

    let one = CodeArray {
        codes: vec![3, 5, 6, 11],
    };
    let two = CodeArray {
        codes: vec![15, 17, 18, 19],
    };
    let three = CodeArray {
        codes: vec![21, 22, 23, 25],
    };
    let four = CodeArray {
        codes: vec![],
    };
    let five_minus = CodeArray {
        codes: vec![],
    };
    let five_plus = CodeArray {
        codes: vec![],
    };
    let six_minus = CodeArray {
        codes: vec![],
    };
    let six_plus = CodeArray {
        codes: vec![],
    };
    let seven = CodeArray {
        codes: vec![],
    };
    
    let quake_prefecture_data = quake_prefecture::QuakePrefectureData {
        rendering_width,
        quaked_unixtime,
        epicenter: Some(epi),
        magnitude_x10: Some(magnitude_x10),
        code_table_difinition: code_table_difinition.to_string(),
        one: Some(one),
        two: Some(two),
        three: Some(three),
        four: Some(four),
        five_minus: Some(five_minus),
        five_plus: Some(five_plus),
        six_minus: Some(six_minus),
        six_plus: Some(six_plus),
        seven: Some(seven),
    };

    let buf = quake_prefecture_data.encode_to_vec();
    wr_b65(&buf)

}

fn wr_file(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("qpd.pr")?;
    file.write_all(buf)?;
    Ok(())
}

fn wr_b65(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = base65536::encode(buf, None);
    println!("{}", encoded);
    Ok(())
}