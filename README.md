# SeccampConnect2026
Security Camp Connect 2026

## クイックスタート（開発・書き込み手順）

開発には Docker (ビルド用) とホスト側の `espflash` (書き込み用) を使用します。
アーキテクチャの図解やコマンドの詳細は [WORKFLOW.md](WORKFLOW.md) を参照してください。

### 1. 開発用コンテナの起動
バックグラウンドでビルド用コンテナを待機させます。
```powershell
docker compose up -d
```

### 2. ビルドの実行
起動中のコンテナ内で `cargo build` を実行します。
```powershell
docker compose exec build cargo build --release
```

### 3. デバイスを「書き込みモード」にする（重要！）
ESP32-S3に書き込むには、マイコンを**ダウンロードモード**にする必要があります。

1. マイコン基板上の **BOOTボタン** を押しっぱなしにする
2. PCにUSBケーブルを接続する（接続済みの場合は、BOOTを押しながらRESETボタンを押して離す）
3. BOOTボタンを離す

### 4. ファームウェアの書き込み
割り当てられたCOMポート（例：`COM3`）を確認して書き込みます。
```powershell
# ポート確認
Get-CimInstance Win32_SerialPort | Select-Object Name, Description

# フラッシュ（COM3の場合）
espflash flash .\target\xtensa-esp32s3-none-elf\release\SeccampConnect2026 --port COM3
```
