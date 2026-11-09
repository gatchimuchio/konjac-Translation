# 現物観測・保全記録 G0

- 観測日：2026-11-09 JST
- 対象：`gatchimuchio/konjac-Translation`
- 状態：**Konjac・依存ソース・添付正本の現物を確認済み。設計整合G1は進行中。**

## Konjac checkout

| 項目 | 観測 |
|---|---|
| Git root / mount | `/workspace/konjac-Translation` / `konjac-Translation` |
| HEAD | `b2da4a15312594571958caaeb5efcc70adb5d8bf` |
| branch | `work` |
| origin identity | `github.com/gatchimuchio/konjac-Translation` |
| origin main | `b2da4a15312594571958caaeb5efcc70adb5d8bf`。当初指定の照合点と一致 |
| history | `b2da4a1`（architecture/governance）→`8d40b42`（repository initialization）の2 commitを確認 |
| submodules | なし |
| source edits | 先行Codex報告にあったJSON Schema、OpenAPI、能力ゲート、13テスト、MCR-001、NOTICE等のファイルを現物で確認。本文・依存版・テストを再照合し、本記録の後段へ状態を分けて記録 |

先行差分を回収できるよう、施工継続前にGit差分と未追跡ファイルを `/tmp/konjac-g0-audited-working.patch` と `/tmp/konjac-g0-audited-untracked.tar.gz` に保全した。保全時SHA-256は順に `b9cc027f9f4ad7b8e1b39b9734d836b4d5ab1a1afb5d028098fc8d1a825a046e`、`9bbe214a02a9ec5f1fa749a5fc1022c29142470a234a9521db1bbf69ec499c4c`。`/tmp`内の保全物は公開成果物・製品証拠ではない。

## 施工正本

Owner提供ZIPの原文3ファイルを [`implementation-baseline/`](implementation-baseline/README.ja.md) にバイト同一で保存した。同ページにZIPと各原文のSHA-256を記録した。正本は製品目的、E01–E06、U01–U12、A01–A07、G0–G8、W01–W12、Rust共通基盤、Tauri 2比較候補、AICompiler移管条件を定義する。

## 依存観測

| 依存 | 版／commit | 観測・権利 | Konjacへの導入 |
|---|---|---|---|
| [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA) | live HEAD `1c546e77007afa0a9eeb3a44f07da99d84c383eb`。凍結 `0.6.0b1` source baseline `fe5822de63644bdaea9208a98046f71684dffae2`。上流 `NOTNN-LLM-MINIDORA@9307d7e2b1799750ecef1957d85128fcbe403d02` | 公開入口、配布マニフェスト、公開境界、R2設計・検証記録を読み、実コードと制御配線例を照合。公開ファサードは任意作用を実行する一般入口で、翻訳言語ペア／文脈継続／翻訳ストリームAPIを確認できない。ソフトウェアはApache-2.0、説明文書はCC-BY-4.0の分別。SudachiPy 0.6.11とSudachiDict-core 20260723はApache-2.0表記、辞書由来物は別表示を確認 | 読取監査のみ。Konjacの製品venvへは未導入。凍結リポへ書込みなし |
| [RTranslator 3.0](https://github.com/niedev/RTranslator/tree/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e) | `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e` (`v3.00`) | サービス／録音／VAD、会話UI、Bluetooth、Android TTS、権限、資源境界とLICENSE/NOTICEを確認。`3.0.0-beta2`、min SDK 28、arm64-v8aを観測。Apache-2.0 | 責務基盤として選定、ソース・モデル・依存・素材は未導入 |
| [Haas-ASR](https://github.com/third-intelligence/Haas-ASR/tree/acc614f68769dcbfae9dba96b48439c4a66295af) | `acc614f68769dcbfae9dba96b48439c4a66295af` | LICENSE、README、短尺／長尺推論、音声分割を確認。ASRモデルとHaas-ja-v1.0重みはREADME上Apache-2.0。NVIDIA GPU前提。疑似ライブ例は定期再認識でありストリーミング実装保証ではない。NeMo等の依存・データ条件は個別 | 参照のみ。ソース・重み・データを導入せず、性能値を製品値に転用しない |
| [speech.cpp](https://github.com/nyosegawa/speech.cpp/tree/03555100287c118b41d1b475044e5ab1e07baa8c) | `03555100287c118b41d1b475044e5ab1e07baa8c` | LICENSE、README、C API／要求・取消コードを確認。ソースMIT。モデル別条件はApache-2.0、MIT＋倫理制約、Gemma Terms、Fish Audio Community、OpenMDW、CC-BY等に分かれる | 参照のみ。コード・モデル・素材を導入していない |

これら3参照元およびMINIDORAのソース・モデルをKonjacへコピーしていない。HDS非公開内容をこの記録に転記していない。

## 開発環境

| 項目 | 観測 |
|---|---|
| OS / architecture | managed Linux / `x86_64-unknown-linux-gnu` |
| Rust / Cargo | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` / `cargo 1.99.1`。公式rustup-initのSHA-256を検証して `/workspace/.rustup` と `/workspace/.cargo` に導入 |
| Python / Node | Python 3.12.14、Node v24.22.1 |
| Python contract venv | `/workspace/.venvs/konjac-translation`; pinned dev dependency checkはPASS |
| 未導入確認 | dotnet、Android SDK/adb、Gradle、FlutterはG0時点で不在。Xcode/macOS SDKはこのLinux環境では未確認／利用不可。Tauri用のLinux system librariesも未確認 |
| permissions/network | Cloud statusはconnected、network policy enforced。GitHub、PyPI、static.rust-lang.orgは実際のread-only取得／TLS検証済み |

Rustホスト環境があることはWindows、macOS、Android、iOS、WebAssembly各ターゲットのbuild/QA成立を意味しない。

## 先行差分の観測と検証状態

- 現物のJSON Schemaは11種のレコード射影、OpenAPI 3.1はHTTP射影、Python能力ゲートと13件のテストが存在する。これらはOwner正本が定める論理契約全体・Rust実装・全環境配線ではない。
- Python 3.12の契約検査は13件PASS、`pip check`とOpenAPI検証もPASS。能力CLIは製品venv内のMINIDORA未導入に対し `UNVERIFIED/MINIDORA_NOT_INSTALLED`。監査用別venvで固定βを確認したプロファイルは翻訳を `UNSUPPORTED` とする。
- JSON Schema/OpenAPIを論理正本と見なした旧記述はOwner正本に合わせて改訂中。論理モデル文書に現在の射影欠落を記録した。
- MCR-001は実ファイルにあり、独立版への変更要求として**記録済み・未提出**。外部へ送信していない。
- 製品HTTPサービス、永続化、W01-W12の全構成、E01-E06の実装／結線／QAは未成立。
