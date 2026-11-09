# こんにゃく翻訳 — ソフトウェア構成設計

- 制定：2026-11-04
- 対象：**翻訳製品の外側**。MINIDORA自身の再設計書ではない。
- 状態：現行正本は[Owner提供の三点セット](implementation-baseline/README.ja.md)。共通論理レコード・同意評価・閉鎖EnginePort境界のRust初期実装、JSON Schema/OpenAPI射影、Python能力検査器は存在する。MINIDORA実行アダプター、媒体アダプター、サービス、UI、永続化、全線結線は未実装。

## 1. 全体構造

```mermaid
flowchart TB
  A["翻訳体験<br/>Desktop / Web / Mobile / API"] --> B["Konjac 入力アダプター<br/>音声・文書・Web・映像・画像・会話"]
  B --> C["媒体前処理<br/>ASR/VAD・OCR・抽出・整列"]
  C --> D["MINIDORA-BETA<br/>入力機 → 本体 → 出力機"]
  D --> E["Konjac 出力整形・配信<br/>字幕・音声・文書・重畳・共有"]
  E --> A
```

外側の **Konjac** は媒体と人間の体験を成立させる。内側の **MINIDORA-BETA** は決められた公開境界のとおりに入力を受け、駆動し、出力する。上流下流のいずれも**翻訳の意味判定・学習・最終採否の第二機構を作らない**。

横断機能：認証・同意・権限、翻訳セッション、ソースと結果の関係付け、保存と消去、監査、環境検査、負荷制御、性能測定。これらは別の意味エンジンではない。

## 2. 推奨パッケージ境界（将来実装の配置案）

| パッケージ候補 | 所有する責務 | 所有しない責務 |
|---|---|---|
| `apps/desktop` | Windows/Linux/macOSのUI、OS権限連携、音声／画面選択、出力表示 | 翻訳の意味判定 |
| `apps/web` | 翻訳Web UI、ブラウザ許可、表示・編集・共有 | ブラウザ権限外の録音／OS強制介入 |
| `apps/mobile` | Android/iOS UI、会話・撮影・共有、権限処理 | 端末制約の迂回 |
| `services/gateway` | セッション、ジョブ受理、認証、制限、配送、明示された遠隔処理 | 任意の自動実行権限 |
| `crates/konjac-core` / `contracts` | 言語・話者・時刻・版を含む論理データ型、状態・能力・同意契約と交換射影 | 自律的採否 |
| `packages/source-adapters` | Mic/system audio/file/camera/web/meeting/host appからの受理 | 他者の通信の無許可取得 |
| `packages/perception` | VAD、ASR、OCR、抽出、話者区別、分割、時間・領域座標 | 言語の意味を独自に翻訳 |
| `packages/translation-bridge` | MINIDORA-BETA公開I/Oに形式変換、能力照会、エラー伝搬、版固定 | 内側アルゴリズム、隠れた代替LLM |
| `packages/presentation` | テキスト、字幕、TTS、吹替、画像重畳、文書再構成 | 生成内容の黙った解釈変更 |
| `packages/sessions` | セッションごとの言語設定、同意、作業状態、元データへの参照 | 中核の学習状態の複製 |
| `packages/provenance` | 原文／訳文の対応、変更履歴、監査、テスト証拠 | 個人情報の無制限保持 |

パッケージ名は責務配置案であり、空のディレクトリを大量作成して「実装完了」としない。Owner提供の現行選定ではRustを共通処理基盤の第一候補、Tauri 2をUI比較基準とする。Tauri 2の各OS音声・画面取得、配布、性能への適合は未検証であり、UI採用は未確定。

## 3. 入口の共通化

1. 取り込み権限と対象の帰属・利用許可を確認。
2. 原媒体（`SourceAsset`）を参照し、形式・時刻基準・言語ヒント・話者候補・配置座標などの**非意味情報**を保管。
3. 入力形式ごとに抽出：音声→ASR/VAD、画像→OCR、動画→音声＋字幕トラック、文書→テキスト＋段落／表／図の対応、Web→DOM／選択範囲。
4. `TranslationRequest` に揃え、MINIDORA公開入力機へ渡す。内容を抽出できなければ原文保持のまま失敗／再試行／利用者修正を提示。
5. `TranslationResult` の出所と成否を検証し、確定状態に応じて表示へ配送。

ASR/OCRは**観測・記号化**の前処理。推測で原文を補い、翻訳済みとして確定させない。

## 4. 三つの代表経路

### A. 会話／同時字幕

```mermaid
flowchart LR
  A["マイク／許可された音声"] --> B["VAD・ASR・話者／時刻"]
  B --> C["MINIDORA-BETA"]
  C --> D["暫定字幕／確定訳"]
  D --> E["画面／任意TTS"]
```

- 暫定文字起こし・暫定訳・確定訳・中止を混同しない。
- ASR区間確定後に訳の確定可否を照合し、古い暫定イベントで確定表示を上書きしない。
- 相手の発話を翻訳音声で流す際、再生音のマイク再入力、音声衝突、話者混同を防ぐ。
- 遅延と字幕品質のトレードオフは計測し、未知なら即時性を保証しない。

### B. Web・PDF・文書

```mermaid
flowchart LR
  A["Web／文書"] --> B["抽出＋構造マップ"]
  B --> C["MINIDORA-BETA"]
  C --> D["訳文／構造との照合"]
  D --> E["表示・原文比較／書き出し"]
```

- DOM、段落、リンク、注、表、画像キャプションなどを翻訳元へ追跡可能とする。
- 文章だけ翻訳して配置破壊や注記喪失を正常終了とみなさない。
- レイアウト非対応の場合は安全な別表示へ明示的に切り替える。

### C. 動画・配信・吹替

```mermaid
flowchart LR
  A["許可された映像"] --> B["音声抽出・区間同期"]
  B --> C["MINIDORA-BETA"]
  C --> D["字幕・TTS・同期"]
  D --> E["再生／字幕出力"]
```

- 時間軸の基準、途中シーク、話速、複数話者、字幕修正の版を管理。
- DRM・配信規約・著作権・画面取得制限を検知したら無許可回避しない。
- 動画自体のエンコード・吹替音声合成は出力機構。意味翻訳の別エンジンではない。

## 5. 共通イベント・状態管理

同一セッションで `source_asset_id`、`session_id`、`utterance_id`／`span_id`、`request_id`、`input_version`、`result_version` を追跡。リアルタイムでは `PROVISIONAL`／`FINAL` を表示状態として区分するが、**MINIDORA内部の採否結果と同一視しない**。

- 再処理は元入力の識別と版を保持。確定済みの訳を修正する場合は新しい版を作る。
- 一つの入力から複数言語への分岐は個々の出力先に対応する相関IDで管理。
- 出力先ごとの送信権限は独立。画面表示が許可されてもチャット送信やクラウド保存が許可されたとは限らない。
- 詳細は[境界契約](ENGINE_INTEGRATION.ja.md)。

## 6. 技術選定と実機前提

- **Windows優先**：デスクトップ、オーディオ装置・仮想音声、スクリーン、Web連携の受入を実機で確かめる。WSL/Linuxだけの成功は代用不可。
- **クロスプラットフォーム**：Windows/Linux/macOS/Android/iOS/Webの全6環境を対象とする。論理契約は共通だが、それぞれのOS・API・配信元権限に従う。
- **ローカルとリモート**：通信先、保存先、依存するモデル／サーバー／ネットワークを実際の処理前に表示できるようにする。自動クラウド流出は禁止。
- **Android外部基盤**：RTranslator 3.0 の `v3.00` ブランチ `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e` を採用方針の基準とする。サービス／音声／会話UI／Bluetooth／システムTTS／権限／汎用資源管理を責務単位で移植対象とする。ソース、モデル、依存はまだ取り込んでいない。RTranslatorの翻訳器と翻訳モデルはMINIDORAの駆動境界から除外する。詳細は[参照ソフトウェア解析](SOURCE_REVIEW.ja.md)。
- **他プラットフォーム**：RTranslatorのAndroid実装をWindows/Web/iOSへ流用扱いしない。共通の[ワイヤ契約](../contracts/konjac-v1.schema.json)に接続するOS固有アダプターを別途施工する。
- **論理契約と射影**：論理レコード・状態・責務を製品正本とし、JSON Schema/OpenAPI/Protobuf/ABI等は必要に応じて生成・維持する交換形式の射影とする。現行JSON Schema/OpenAPIは実装済みの候補射影で、論理モデルの全同一性・全状態をまだ表現しきっていない。詳細は[論理契約](../contracts/LOGICAL_MODEL.ja.md)。
- **Rust共通基盤**：Owner提供の現行仕様に従いRustを施工第一候補とする。共通処理・状態機械・EnginePort・資源制御に適用し、OS固有I/Oはアダプター境界に置く。Rustの選定はAICompilerによる将来互換の証明ではなく、ビルド・ターゲット・依存・性能・配布適合の検証を要する。
- **UI候補**：Tauri 2をWindows/Linux/macOSのデスクトップと共有UI候補の比較基準とする。E01-E06の音声、画面、ファイル、権限、署名、配布、WASM適合を未検証のまま採用確定しない。Web、Android、iOSに必要なTypeScript/Kotlin/Swift等は同一論理契約の境界付き射影とする。
- **限定したPython選定**：Python `>=3.11` はMINIDORA公開ファサードの能力検査CLIと契約QCに限る。Pythonを製品共通ランタイムへ拡張しない。JSON Schema/OpenAPI検証器はテスト用である。

## 7. 未解決（最初の実装で検証）

1. MINIDORA-BETAの公開契約で、指定元言語・複数訳先・継続文脈・短遅延処理を満たせるか。**未実証**。
2. ASR/OCR/TTSの端末別コスト、対応言語、語彙、レイテンシ、外部API要否。
3. Windows上の安全かつ合法な他アプリ音声取得と字幕重畳の手法。
4. 文書構造・動画時間軸・翻訳版の一貫性を測る受入指標。
5. モデル配布、ライセンス、端末容量、キャッシュ・権限・秘密管理。

未解決事項は仕様から消すのではなく、[工程](DELIVERY_PLAN.ja.md)の検証ゲートへ渡す。

## 8. 現在の共通コア実装範囲

`crates/konjac-core`には7中核record、ConsentGrant/ConsentRevocation、TranslationResult/TranslationEventの型、BCP 47 well-formedタグ構文と正規化、DeliveryArtifact向けの別個なDeliver scope検査と対象artifactへ束縛したpermit、基本不変条件、明示的同意評価（重複grant IDは拒否）、アクティブsession/request/input版とSourceAsset/Spanの参照検査、訳先出力の全数参照検査を実装した。`AuthorizationContext`を使う経路は、認証済み主体のgrantがSourceAssetごとのCaptureとProcessRemote/ProcessLocalを許可し、期限内で撤回されていないことを照会前に検査する。DeliveryPermitは使用時に同じartifactとgrantを再検証する。EnginePortの能力許可は原言語・訳先・文脈要否・batch/streamingと版付き証拠を束縛する。`execute_guarded_at`は有効な期限・取消時刻を比較し、能力照会前に先に成立した終端理由で停止する。認証サービス、永続grant台帳、配送アダプターは未実装。Rust内の`contract_version`は論理契約版であり、JSON Schemaの`schema_version`と同じ物として扱わない。

このcrateには翻訳モデル・意味判断ロジック・MINIDORA呼出し実装はない。許可評価は渡された同意記録に対する純関数であり、永続台帳、OS権限、認証、実際のデータ消去を代替しない。期限・取消検査はエンジン呼出し前に限られ、実行中の取消伝播と停止確認を実装しない。event batch検査は読み込んだpageの整合を確認するが、DBのsequence採番/追記を保証しない。Rust単体テストは構造/境界の一部を確認するだけで、W01/W02/W03全体の実装、結線、QCをPASSにしない。論理型からJSON/OpenAPIへの完全射影も未施工。

