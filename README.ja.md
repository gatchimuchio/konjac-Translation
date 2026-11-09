# こんにゃく翻訳 — Konjac Translation

[English](README.md) · [製品仕様](docs/PRODUCT_SPEC.ja.md) · [アーキテクチャ](docs/ARCHITECTURE.ja.md) · [開発工程](docs/DELIVERY_PLAN.ja.md)

**目標は「言語をフリーにする」体験を、生活・仕事・コンテンツのあらゆる翻訳場面へ提供すること。**

こんにゃく翻訳（プロジェクト名 `konjacTranslation`）は、**文字・音声・映像・画像・文書・Web・アプリ・通話・会議を横断するマルチ翻訳プラットフォーム**である。文字起こしアプリ、同時通訳アプリ、字幕アプリの寄せ集めを最終形としない。

- **読む**：任意テキスト、Web、PDF、Office文書、画像・看板・スクリーンショットを翻訳する。
- **話す／聞く**：対面会話、録音、電話・オンライン会議を翻訳し、字幕・テキスト・音声で届ける。
- **見る**：動画・配信・教材・会議映像を文字起こしし、多言語字幕や翻訳音声で利用する。
- **書く／伝える**：チャット、投稿、メール、文書を相手の言語へ変換する。
- **つなぐ**：Windows、Linux、macOS、Android、iOS、Web、許可された外部アプリ、API/SDKで同じ論理契約の翻訳体験を提供する。

## 体験の原則

1. 言語設定・人物・文脈・専門用語を入力経路ごとに何度も設定させない。
2. 同じ翻訳の結果を、文字・字幕・音声・文書・画面重畳に切り替えて届けられる。
3. 原文と訳文の対応、話者、時刻、版、修正履歴を保持する。情報を黙って書き換えない。
4. 入力元アプリや媒体の切替で翻訳の意味・関係・文脈を落とさない。
5. マイク・カメラ・画面取得・外部送信は、権限と同意を通過した範囲でのみ実施する。

## ミニドラとの責任分離

翻訳を駆動する中核は [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA)。既存の**入力機 → ミニドラ本体 → 出力機**を保つ。

**こんにゃく翻訳が実装するのは、その外側**。音声取得／ASR／VAD、OCR、文章抽出、画面や会議の接続、時間軸・話者情報、翻訳成果物の出力、TTS、UI/UX、記録・権限・配信などである。ミニドラの内部能力や意味判断・採否を独自に作り直さない。

RTranslator 3.0 (`ae96b008e31323ba8b75f94e8b5d4c29b8c1307e`) をAndroid外部基盤として選定した。音声・サービスのライフサイクル、会話UI、Bluetooth、システムTTS、権限、汎用資源管理を責務単位で移植対象とし、同プロジェクトの翻訳器・翻訳モデルは除外する。ソースはまだ取り込んでいない。Haas-ASRとspeech.cppは引き続き解析候補である。[採用境界](docs/SOURCE_REVIEW.ja.md)。

## 現在の状態

※ **2026-11-09時点：Rust共通コアに論理レコード、同意評価、閉鎖的なEnginePort境界を実装し、単体検査済み。MINIDORA実行アダプター、翻訳アプリ、HTTPサービス、永続化、媒体/OSアダプターは未実装。** 対応言語・OS・翻訳品質・リアルタイム性能は保証しない。MINIDORA-BETA `0.6.0b1` の公開資料・実装から実訳APIを確認できていない。

Owner提供の現行施工正本（製品仕様・全体工程・Codex指示）は[`docs/implementation-baseline/`](docs/implementation-baseline/README.ja.md)に原文保存している。Windows/Linux/macOS/Android/iOS/Webの6環境を対象とし、G0–G8の全体工程で追跡する。

全体系の要求・境界・契約を保ち、入力・中核・出力・各環境を結線してから全体経路を検収する。A01〜A07、U01〜U12は施工漏れ防止と受入の索引であり、**製品定義を「スマホの同時翻訳＋文字起こし」に縮退させない**。

Rust `1.99.1`を共通処理基盤に採用し、`crates/konjac-core`に論理契約の第一実装を置いた。22件のRust単体テストと15件のPython射影・能力検査が通る。同意grant IDの重複は曖昧として拒否する。Rustのセッション検査はSourceAsset/Spanと各版を結び、能力許可は原言語・訳先・文脈要否・batch/streamingの組合せに束縛する。要求時間窓を検査し、`UNSUPPORTED`、期限切れ、取消済み要求はエンジン照会前に停止する。Tauri 2はデスクトップ／共有UIの比較候補であり採用確定ではない。JSON Schema/OpenAPIは論理契約から導く交換射影で、完全な対応は未検証。

開発環境の確認コマンド：

```sh
python -m pip install -r requirements-dev.lock
python -m unittest discover -s tests -v
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

- [製品仕様](docs/PRODUCT_SPEC.ja.md)
- [構成／モジュール境界](docs/ARCHITECTURE.ja.md)
- [ミニドラ接続契約](docs/ENGINE_INTEGRATION.ja.md)
- [既存ソフト解析](docs/SOURCE_REVIEW.ja.md)
- [工程・受入基準](docs/DELIVERY_PLAN.ja.md)、[受入検証](docs/ACCEPTANCE.ja.md)
- [クラウド開発環境](docs/CLOUD_ENVIRONMENT_SETUP.ja.md)
- [開発規律](AGENTS.md)、[寄稿方針](CONTRIBUTING.md)、[ライセンス](LICENSE)、[第三者帰属](NOTICE)

著者：**がっちむち♂（gatchimuchio）**。

