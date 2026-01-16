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

|Versioned Type ID|Protobuf File|
|:--|:--|
|`0`|`quake_prefecture_v0.proto`|
|`1`|`tsunami_v0.proto`|
