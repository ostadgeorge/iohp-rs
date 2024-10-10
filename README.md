## Iranian Oral History Project

- install geckodriver and firefox
```bash
sudo pacman -S geckodriver firefox
./geckodriver
```
- scrap raw data
```
cargo run --bin scrap
```
- extract items from raw data
```
cargo run --bin extract
```
- scrap audio m3u8
```
cargo run --bin scrap-audio
```
- download audio
```
cargo run --bin download-audio
```