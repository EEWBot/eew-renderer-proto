# eew-renderer-proto

## v0 (base65536 encoded)

```
[ 0.. 1] Version                             (1  byte) [Always: 0]
[ 1..21] HMAC<Sha1>                          (20 bytes)
[21..  ] Body quake_prefecture_v0 (Protobuf) (N bytes)
```
