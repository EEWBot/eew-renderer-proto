pub mod quake_prefecture {
    include!(concat!(env!("OUT_DIR"), "/quake_prefecture.rs"));
}

use prost::Message;

use serde::Deserialize;
use serde_json;


impl<'de> Deserialize<'de> for quake_prefecture::QuakePrefectureData {
    fn deserialize<D>(deserializer: D) -> Result<quake_prefecture::QuakePrefectureData, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let qpd: quake_prefecture::QuakePrefectureData = serde_json::from_str(&s).unwrap();
        Ok(qpd)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = include_str!("20240808075502_0_VXSE53_010000.json");
    let qpd: quake_prefecture::QuakePrefectureData = serde_json::from_str(target)?;

    let encoded_buf = base65536::encode(&qpd.encode_to_vec(), None);
    print!("{}", encoded_buf);
    Ok(())

}