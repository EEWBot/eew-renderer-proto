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
```rust
use num_bigint::BigUint;
use num_traits::Zero;

pub const MAX_SYMBOL: u8 = 9;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CodecError {
    #[error("empty payload")]
    Empty,

    #[error("unknown mode byte: {0}")]
    UnknownMode(u8),

    #[error("truncated payload")]
    Truncated,

    #[error("varint overflows u64")]
    VarintOverflow,

    #[error("symbol value {0} out of range (max {MAX_SYMBOL})")]
    SymbolOutOfRange(u16),
}

const MODE_RLE: u8 = 0;
const MODE_BASE10: u8 = 1;

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

    let rle = encode_rle(symbols);
    let base10 = encode_base10(symbols);

    let out = if rle.len() <= base10.len() {
        let mut out = Vec::with_capacity(1 + rle.len());
        out.push(MODE_RLE);
        out.extend_from_slice(&rle);
        out
    } else {
        let mut out = Vec::with_capacity(1 + base10.len());
        out.push(MODE_BASE10);
        out.extend_from_slice(&base10);
        out
    };

    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeResult {
    /// renderer が知っている観測点の震度。
    pub symbols: Vec<u8>,
    pub has_unknown_tail: bool,
}

pub fn decode(data: &[u8], known_station_count: usize) -> Result<DecodeResult, CodecError> {
    let (&mode, body) = data.split_first().ok_or(CodecError::Empty)?;

    let (mut symbols, has_unknown_tail) = match mode {
        MODE_RLE => decode_rle(body, known_station_count)?,
        MODE_BASE10 => {
            let symbols = decode_base10(body)?;
            let has_unknown_tail = symbols.len() > known_station_count;
            (symbols, has_unknown_tail)
        }
        other => return Err(CodecError::UnknownMode(other)),
    };

    symbols.truncate(known_station_count);
    symbols.resize(known_station_count, 0);

    Ok(DecodeResult {
        symbols,
        has_unknown_tail,
    })
}

const TOK_ZERO_RUN: u8 = 0x00;
const TOK_VALUE_RUN: u8 = 0x0A;
const VALUE_RUN_MIN: usize = 3;

fn encode_rle(symbols: &[u8]) -> Vec<u8> {
    debug_assert!(symbols.iter().all(|&v| v <= MAX_SYMBOL));

    let mut out = Vec::new();
    let mut i = 0;
    while i < symbols.len() {
        let v = symbols[i];
        let mut j = i + 1;
        while j < symbols.len() && symbols[j] == v {
            j += 1;
        }
        let run = j - i;

        if v == 0 {
            out.push(TOK_ZERO_RUN);
            put_varint(&mut out, run as u64);
        } else if run >= VALUE_RUN_MIN {
            out.push(TOK_VALUE_RUN);
            out.push(v);
            put_varint(&mut out, run as u64);
        } else {
            for _ in 0..run {
                out.push(v);
            }
        }
        i = j;
    }
    out
}

fn decode_rle(mut body: &[u8], known_station_count: usize) -> Result<(Vec<u8>, bool), CodecError> {
    let mut out = Vec::new();
    let mut pos = 0u64;
    let mut has_unknown_tail = false;

    while let Some((&tok, rest)) = body.split_first() {
        let (value, run, rest) = match tok {
            TOK_ZERO_RUN => {
                let (run, rest) = get_varint(rest)?;
                (0, run, rest)
            }
            1..=9 => (tok, 1, rest),
            TOK_VALUE_RUN => {
                let (&v, rest) = rest.split_first().ok_or(CodecError::Truncated)?;
                if v == 0 || v > MAX_SYMBOL {
                    return Err(CodecError::SymbolOutOfRange(v as u16));
                }
                let (run, rest) = get_varint(rest)?;
                (v, run, rest)
            }
            other => return Err(CodecError::SymbolOutOfRange(other as u16)),
        };

        push_run(
            &mut out,
            &mut pos,
            &mut has_unknown_tail,
            value,
            run,
            known_station_count,
        );
        body = rest;
    }

    Ok((out, has_unknown_tail))
}

fn push_run(
    out: &mut Vec<u8>,
    pos: &mut u64,
    has_unknown_tail: &mut bool,
    value: u8,
    run: u64,
    known_station_count: usize,
) {
    let known = known_station_count as u64;
    let end = pos.saturating_add(run);

    if value != 0 && end > known {
        *has_unknown_tail = true;
    }

    let visible = known.saturating_sub(*pos).min(run) as usize;
    out.extend(std::iter::repeat_n(value, visible));
    *pos = end;
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

fn put_varint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if v == 0 {
            break;
        }
    }
}

fn get_varint(data: &[u8]) -> Result<(u64, &[u8]), CodecError> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in data.iter().enumerate() {
        if shift >= 64 || (shift == 63 && byte & 0x7f > 1) {
            return Err(CodecError::VarintOverflow);
        }
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, &data[i + 1..]));
        }
        shift += 7;
    }
    Err(CodecError::Truncated)
}
```
