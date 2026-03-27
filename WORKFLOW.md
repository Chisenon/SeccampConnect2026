```mermaid
flowchart TD
    subgraph HOST["Host (Windows)"]
        EDITOR["エディタ <br> src/main.rs を編集"]
        VOLUME[".\\ (プロジェクトルート)"]
        ESPFLASH["espflash<br>(ホスト側)"]
        COM["USB Serial<br>COM3 等"]
        ESP32S3["ESP32S3<br>(XIAO) / BOOTボタンON"]
    end

    subgraph CONTAINER["Docker Container (常時起動)"]
        MOUNT["/work (bind mount)"]
        CARGO["cargo build --release<br>(Xtensa Rust toolchain esp)"]
        ELF["target/xtensa-esp32s3-none-elf <br> /release/SeccampConnect2026"]

        MOUNT --> CARGO
        CARGO --> ELF
    end

    EDITOR -->|"1. ファイル保存"| VOLUME
    VOLUME <-->|"bind mount<br>.:/work"| MOUNT
    ELF -->|"target/ はホストと共有"| VOLUME

    VOLUME -->|"2. docker compose exec build..."| CONTAINER

    VOLUME -->|"4. espflash flash ... --port COM3"| ESPFLASH
    ESPFLASH -->|"USB通信"| COM
    COM -->|"3. ダウンロードモード待機"| ESP32S3
```

## 開発ワークフロー

### 1. 開発環境の起動
DevContainerが言う事聞いてくれないから、
Dockerでビルド用コンテナを手動で起動します。
```powershell
docker compose up -d
```

### 2. ファームウェアのビルド
起動中のコンテナ内でビルドを実行します。
```powershell
docker compose exec build cargo build --release
```

### 3. デバイスを書き込みモード（ダウンロードモード）にする
**BOOTボタンを押しながら**接続してデバイスをPCに認識させる！

### 4. ポートの確認
デバイスが認識されたら、COMポート番号を確認します。
```powershell
Get-CimInstance Win32_SerialPort | Select-Object Name, Description
```

COMX の部分は、実際に認識されたCOMポート番号に置き換えてね！

```
COM3
```

### 5. ファームウェアの書き込み
ホスト側（Windows）から `espflash` を使ってビルドしたELFファイルを書き込みます。
```powershell
espflash flash .\target\xtensa-esp32s3-none-elf\release\SeccampConnect2026 --port COMX
```

---

## コマンド早見表

| ステップ | コマンド |
|---|---|
| コンテナ起動 | `docker compose up -d` |
| コンテナ停止 | `docker compose down` |
| ビルド (release) | `docker compose exec build cargo build --release` |
| ビルドシェルに入る | `docker compose exec build bash` |
| ポート確認 | `Get-CimInstance Win32_SerialPort \| Select-Object Name, Description` |
| 書き込み | `espflash flash .\target\xtensa-esp32s3-none-elf\release\SeccampConnect2026 --port COMX` |
