# eew-renderer-proto

## Legacy format (base65536 encoded)

```
[ 0.. 1] Version                             (1  byte) [Always: 0]
[ 1..21] HMAC<Sha1>                          (20 bytes)
[21..  ] Body quake_prefecture_v0 (Protobuf) (N bytes)
```

## Modern format (base32768 encoded)

```
[ 0.. 1] Versioned type id    (1  byte)
[ 1.. 2] Non base65536 marker (1  byte)  [Always: 0xFF]
[ 2..22] HMAC<Sha1>           (20 bytes)
[22..  ] Body Protobuf        (N bytes)
```

### Versioned Type ID

|Versioned Type ID|Protobuf Message|
|:--|:--|
|`0`|`QuakePrefectureV0`|
|`1`|`TsunamiForecastV0`|
|`2`|`TsunamiForecastV1`|
|`3`|`QuakeWarningV0`|
|`4`|`QuakeStationV0`|


### QuakeStationV0.intensities codec

観測点ごとの震度（canonical な観測点 index 順、各要素 `0..=9`）の可逆コーデック。
先頭 1 バイトの `mode` と本体からなる:

|`mode`|Body|
|:--|:--|
|`0`|zstd（dictionary・checksum・content size frame なし）|
|`1`|シンボル列全体を 1 個の 10 進数とみなしたradix conversion（index 0 が最下位桁）|

依存: `num-bigint` / `num-traits` / `thiserror` / `zstd`

```rust
use num_bigint::BigUint;
use num_traits::Zero;

pub const MAX_SYMBOL: u8 = 9;

pub const MAX_DECOMPRESSED: usize = 1 << 16;

pub const MAX_WINDOW_LOG: u32 = 16;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CodecError {
    #[error("empty payload")]
    Empty,

    #[error("unknown mode byte: {0}")]
    UnknownMode(u8),

    #[error("truncated or corrupt payload")]
    Truncated,

    #[error("decompressed body exceeds {MAX_DECOMPRESSED} bytes")]
    TooLong,

    #[error("symbol value {0} out of range (max {MAX_SYMBOL})")]
    SymbolOutOfRange(u16),
}

const MODE_ZSTD: u8 = 0;
const MODE_BASE10: u8 = 1;

const ZSTD_LEVEL: i32 = 20;

fn trim_trailing_zeros(symbols: &[u8]) -> &[u8] {
    let end = symbols
        .iter()
        .rposition(|&v| v != 0)
        .map_or(0, |last| last + 1);
    &symbols[..end]
}

pub fn encode(symbols: &[u8]) -> Result<Vec<u8>, CodecError> {
    if let Some(&v) = symbols.iter().find(|&&v| v > MAX_SYMBOL) {
        return Err(CodecError::SymbolOutOfRange(v as u16));
    }

    let symbols = trim_trailing_zeros(symbols);

    let base10 = encode_base10(symbols);

    let zstd = (symbols.len() <= MAX_DECOMPRESSED).then(|| encode_zstd(symbols));

    let out = match zstd {
        Some(zstd) if zstd.len() <= base10.len() => {
            let mut out = Vec::with_capacity(1 + zstd.len());
            out.push(MODE_ZSTD);
            out.extend_from_slice(&zstd);
            out
        }
        _ => {
            let mut out = Vec::with_capacity(1 + base10.len());
            out.push(MODE_BASE10);
            out.extend_from_slice(&base10);
            out
        }
    };

    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeResult {
    pub symbols: Vec<u8>,

    pub has_unknown_tail: bool,
}

pub fn decode(data: &[u8], known_station_count: usize) -> Result<DecodeResult, CodecError> {
    let (&mode, body) = data.split_first().ok_or(CodecError::Empty)?;

    let symbols = match mode {
        MODE_ZSTD => decode_zstd(body)?,
        MODE_BASE10 => decode_base10(body)?,
        other => return Err(CodecError::UnknownMode(other)),
    };

    let symbols = trim_trailing_zeros(&symbols);
    let has_unknown_tail = symbols.len() > known_station_count;

    let mut symbols = symbols.to_vec();
    symbols.truncate(known_station_count);
    symbols.resize(known_station_count, 0);

    Ok(DecodeResult {
        symbols,
        has_unknown_tail,
    })
}

fn encode_zstd(symbols: &[u8]) -> Vec<u8> {
    use zstd::zstd_safe::CParameter;

    debug_assert!(symbols.iter().all(|&v| v <= MAX_SYMBOL));
    debug_assert!(symbols.len() <= MAX_DECOMPRESSED);

    let mut compressor =
        zstd::bulk::Compressor::new(ZSTD_LEVEL).expect("zstd level is a valid constant");
    for parameter in [
        CParameter::ChecksumFlag(false),
        CParameter::ContentSizeFlag(false),
    ] {
        compressor
            .set_parameter(parameter)
            .expect("zstd frame parameters are supported");
    }

    compressor.compress(symbols).expect("zstd compression")
}

fn decode_zstd(body: &[u8]) -> Result<Vec<u8>, CodecError> {
    use std::io::Read;

    let mut decoder = zstd::stream::read::Decoder::new(body).map_err(|_| CodecError::Truncated)?;
    decoder
        .window_log_max(MAX_WINDOW_LOG)
        .expect("window log is within the supported range");

    let mut symbols = Vec::new();
    decoder
        .take(MAX_DECOMPRESSED as u64 + 1)
        .read_to_end(&mut symbols)
        .map_err(|_| CodecError::Truncated)?;

    if symbols.len() > MAX_DECOMPRESSED {
        return Err(CodecError::TooLong);
    }

    if let Some(&v) = symbols.iter().find(|&&v| v > MAX_SYMBOL) {
        return Err(CodecError::SymbolOutOfRange(v as u16));
    }

    Ok(symbols)
}

fn encode_base10(symbols: &[u8]) -> Vec<u8> {
    debug_assert!(symbols.iter().all(|&v| v <= MAX_SYMBOL));

    if symbols.is_empty() {
        return Vec::new();
    }

    let digits: Vec<u8> = symbols.iter().rev().map(|d| b'0' + d).collect();

    let value = BigUint::parse_bytes(&digits, 10).expect("digits are always valid base-10");

    if value.is_zero() {
        return Vec::new();
    }

    value.to_bytes_be()
}

fn decode_base10(body: &[u8]) -> Result<Vec<u8>, CodecError> {
    let value = BigUint::from_bytes_be(body);

    if value.is_zero() {
        return Ok(Vec::new());
    }

    Ok(value
        .to_str_radix(10)
        .as_bytes()
        .iter()
        .rev()
        .map(|c| c - b'0')
        .collect())
}
```
