pub mod quake_prefecture {
    include!(concat!(env!("OUT_DIR"), "/quake_prefecture.rs"));
}

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
    println!("hoge");
    
    let quake_prefecture_data = QuakePrefectureData {
        rendering_width: qpd_json.rendering_width,
        quaked_unixtime: qpd_json.quaked_unixtime,
        magnitude_x10: Some(qpd_json.magnitude_x10.unwrap()),
        code_table_definition: qpd_json.code_table_difinition,
        epicenter: Some(Epicenter {
            lat_x10: qpd_json.epicenter.as_ref().unwrap().lat_x10,
            lon_x10: qpd_json.epicenter.as_ref().unwrap().lon_x10,
        }),
        one: Some(CodeArray { codes: qpd_json.one.unwrap() }),
        two: Some(CodeArray { codes: qpd_json.two.unwrap() }),
        three: Some(CodeArray { codes: qpd_json.three.unwrap() }),
        four: Some(CodeArray { codes: qpd_json.four.unwrap() }),
        five_minus: Some(CodeArray { codes: qpd_json.five_minus.unwrap() }),
        five_plus: Some(CodeArray { codes: qpd_json.five_plus.unwrap() }),
        six_minus: Some(CodeArray { codes: qpd_json.six_minus.unwrap() }),
        six_plus: Some(CodeArray { codes: qpd_json.six_plus.unwrap() }),
        seven: Some(CodeArray { codes: qpd_json.seven.unwrap() }),
    };
    println!("hoge");
    
    let buf: Vec<u8> = quake_prefecture_data.encode_to_vec();
    wr_b65(&buf)
}

fn wr_b65(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = base65536::encode(buf, None);
    println!("{}", encoded);
    Ok(())
}
