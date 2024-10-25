pub mod quake_prefecture {
    include!(concat!(env!("OUT_DIR"), "/quake_prefecture.rs"));
}

use std::fs::File;
use std::io::Write;

use prost::Message;
use quake_prefecture::{CodeArray, Epicenter, QuakePrefectureData};

use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct QuakePrefDataEpicenter {
    lat_x10: i32,
    lon_x10: i32,
}

#[derive(Deserialize)]
struct QuakePrefData {
    rendering_width: u32,
    quaked_unixtime: u64,
    epicenter: Option<QuakePrefDataEpicenter>,
    magnitude_x10: Option<u32>,
    code_table_difinition: String,

    one: Option<Vec<u32>>,
    two: Option<Vec<u32>>,
    three: Option<Vec<u32>>,
    four: Option<Vec<u32>>,
    five_minus: Option<Vec<u32>>,
    five_plus: Option<Vec<u32>>,
    six_minus: Option<Vec<u32>>,
    six_plus: Option<Vec<u32>>,
    seven: Option<Vec<u32>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = include_str!("20240808075502_0_VXSE53_010000.json");
    let qpd_json: QuakePrefData = serde_json::from_str(target)?;
    // println!("hoge");

    let quake_prefecture_data = QuakePrefectureData {
        rendering_width: qpd_json.rendering_width,
        quaked_unixtime: qpd_json.quaked_unixtime,
        magnitude_x10: qpd_json.magnitude_x10,
        code_table_definition: qpd_json.code_table_difinition,
        epicenter: qpd_json.epicenter.map(|v| Epicenter {
            lat_x10: v.lat_x10,
            lon_x10: v.lon_x10,
        }),
        one: qpd_json.one.map(|v| CodeArray { codes: v }),
        two: qpd_json.two.map(|v| CodeArray { codes: v }),
        three: qpd_json.three.map(|v| CodeArray { codes: v }),
        four: qpd_json.four.map(|v| CodeArray { codes: v }),
        five_minus: qpd_json.five_minus.map(|v| CodeArray { codes: v }),
        five_plus: qpd_json.five_plus.map(|v| CodeArray { codes: v }),
        six_minus: qpd_json.six_minus.map(|v| CodeArray { codes: v }),
        six_plus: qpd_json.six_plus.map(|v| CodeArray { codes: v }),
        seven: qpd_json.seven.map(|v| CodeArray { codes: v }),
    };
    // println!("hoge");

    let buf: Vec<u8> = quake_prefecture_data.encode_to_vec();
    let zstd_buf = zstd::encode_all(&buf[..], 0)?;
    wr_b65(&zstd_buf)
}

fn wr_b65(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("qpd.txt")?;
    file.write_all(base65536::encode(buf, None).as_bytes())?;
    Ok(())
}
