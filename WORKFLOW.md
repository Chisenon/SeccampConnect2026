```mermaid
flowchart TD
    subgraph HOST["Host"]
        EDITOR["エディタ <br> src/main.rs を編集"]
        VOLUME[".\\ (プロジェクトルート)"]
        ESPFLASH["espflash<br>(ホスト側)"]
        COM["USB Serial<br>COM3"]
        ESP32S3["ESP32S3<br>(XIAO)"]
    end

    subgraph CONTAINER["Docker Container"]
        MOUNT["/work (bind mount)"]
        CARGO["cargo build --release<br>(Xtensa Rust toolchain esp)"]
        ELF["target/xtensa-esp32s3-none-elf <br> /release/ESP32S3_LED_blink"]

        MOUNT --> CARGO
        CARGO --> ELF
    end

    EDITOR -->|"ファイル保存"| VOLUME
    VOLUME <-->|"bind mount<br>.:/work"| MOUNT
    ELF -->|"target/ はホストと共有"| VOLUME

    VOLUME -->|"docker compose run --rm build"| CONTAINER

    VOLUME -->|"espflash flash ... --port COM3"| ESPFLASH
    ESPFLASH --> COM
    COM --> ESP32S3
```

## コマンド早見表

| ステップ | コマンド |
|---|---|
| ビルド (release) | `docker compose run --rm build` |
| ビルド (debug) | `docker compose run --rm build cargo build` |
| ポート確認 | `Get-PnpDevice -Class Ports \| Where-Object Status -eq 'OK' \| Select-Object FriendlyName` |
| 書き込み | `espflash flash .\target\xtensa-esp32s3-none-elf\release\ESP32S3_LED_blink --port COM3` |
| コンテナに入る | `docker compose run --rm --entrypoint bash build` |
