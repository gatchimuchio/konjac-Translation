# MINIDORA-BETA 接続・データ契約

- 更新：2026-11-09
- 対象：MINIDORA Engine Beta 1 `0.6.0b1`
- 状態：公開APIを監査し、Rust論理契約・JSON/OpenAPI射影・能力ゲート境界を実装。MINIDORA実訳接続は **BLOCKED**。

この契約はこんにゃく翻訳が所有する入出力・来歴・権限の境界を定める。MINIDORA内部の意味処理、学習、判断、採否を再定義しない。

## 1. 監査した依存の正本

2026-11-09にGitHubから read-only clone した `gatchimuchio/MINIDORA-BETA` のHEADは `1c546e77007afa0a9eeb3a44f07da99d84c383eb`。`0.6.0b1` の凍結配布基準は `RELEASE_MANIFEST.json` の `fe5822de63644bdaea9208a98046f71684dffae2` と、元上流 `gatchimuchio/NOTNN-LLM-MINIDORA@9307d7e2b1799750ecef1957d85128fcbe403d02`。GitHub現行HEADには後続の文書更新があるが、βの実行ソースは変更されていない。

| 確認項目 | 観測した事実 |
|---|---|
| 配布名・版 | `minidora-notnn==0.6.0b1`、Python `>=3.11` |
| 依存 | `SudachiPy==0.6.11`、`SudachiDict-core==20260723` |
| 外部ファサード | `minidora.engine_beta.ミニドラβ実行機` |
| 公開操作 | `実行(...)`、同一インスタンス内の `結果を参照(...)` |
| 一般入力 | テキスト本文。`案件ID`、`入力ID`、入力版等を受け取る |
| 例の実行方式 | 呼出側から作用を注入する制御配線例。指定した作用の出力を返す |
| 実訳能力 | 公開API・配布記録から言語ペア翻訳の契約を確認できない |
| ストリーミング | 翻訳イベント／暫定訳／確定訳の公開契約を確認できない |
| 永続性 | 例の重複抑止は同一ファサードインスタンス内。再起動を越える保証はない |
| 配信 | `実行()` は外部送信を行わない |

固定βのAPI・公開境界・配布ルールを変更しない。2026-11-09時点の公開HEAD `1c546e77007afa0a9eeb3a44f07da99d84c383eb` で `docs/redesign/README`、`01_SYSTEM_SPECIFICATION`、`05_VERIFICATION_AND_EVIDENCE`、`07_R1_AUDIT_CLOSURE` も照合した。R2文書は後継設計・検証計画であり、実装／公開経路の受入が保留であることを区別している。R2用の設計記述や検証項目は独立した実訳API・実行証拠ではなく、凍結βの能力を拡張しない。また同資料はPython・SQLite・JSONを論理構成の必須選択とはしていないため、Konjacの全体技術選定根拠として扱わない。R2の詳細理論はここへ転記しない。

## 2. 統合状態と停止動作

`0.6.0b1`には、確認できた公開契約として `translate(text, from, to)` 相当の操作がなく、言語ペア・文脈継続・翻訳キャンセル・翻訳ストリーミングも契約されていない。制御例は固定の入力→作用→出力を確認するもので、実訳能力の試験ではない。固定βと公開資料を監査した結論は、こんにゃく翻訳の実訳経路を接続できる証拠がまだないということ。

したがって対象要求は `UNSUPPORTED`、`SUSPEND` または原因に応じた `FAILED` として記録し、訳文を生成・配信しない。ASR/OCR/TTS、RTranslator内の翻訳モデル、外部翻訳APIをMINIDORAの代替成功として隠して呼ばない。任意言語ペア翻訳は `BLOCKED` の受入項目として保持する。新しいMINIDORA公開APIや版が必要なため、凍結βを改変せず独立変更要求 [MCR-001](MINIDORA_CHANGE_REQUEST.ja.md) を記録した。未提出であり、後継版・承認・実装証拠はない。

## 3. 実装済みの共通ワイヤ契約

論理契約の正本は[`contracts/LOGICAL_MODEL.ja.md`](../contracts/LOGICAL_MODEL.ja.md)、初期Rust型は[`crates/konjac-core`](../crates/konjac-core)に置く。[`contracts/konjac-v1.schema.json`](../contracts/konjac-v1.schema.json)はJSON交換射影で、Draft 2020-12を製品論理の正本にはしない。全項目の射影同値は未施工。Python能力ゲートは [`src/konjac_translation/engine_profile.py`](../src/konjac_translation/engine_profile.py) と監査済み版プロファイルを読み、未監査版を `UNVERIFIED` とする。Rust `EnginePort`は原言語・訳先・文脈要否・batch/streamingを指定する型付き照会と、厳密なエンジン版・commit・公開入口の証拠が一致したときだけ閉鎖型実行許可を作る。`execute_guarded`は入力・セッション・期限・取消を検査し、認証済み主体を含むAuthorizationContextが渡された場合は媒体取得と処理の同意範囲・対象・期限・撤回を照会前に確認する。その後、全照会の事前判定を経てから単一エンジンを呼び、拒否・未知が一つでもあれば呼出しを止める。期限と取消が両方成立していれば、先に成立した状態を返す。返却後は許可集合、実行証拠、各訳先出力の対応を検査する。実行中の取消伝播と停止確認、認証サービスは未実装。これはMINIDORA実行アダプターではなく、翻訳実行も行わない。

`schema_version` は現在 `1.0.0`。型と責務は次のとおり。

| レコード | 責務 |
|---|---|
| `SourceAsset` | 原媒体の種別、取得経路、参照、媒体型、保持方針 |
| `TranslationSession` | 入出力言語、参加者、共有文脈・用語、処理場所、保持方針 |
| `ConsentGrant` / `ConsentRevocation` | 主体・対象・操作範囲・明示許可、および履歴を残す独立した撤回記録 |
| `SourceSpan` | 原文、言語候補、話者、観測方法、時刻／文書位置、来歴 |
| `TranslationRequest` | session/request/input各版、訳先集合、文脈、出力形式、実行モード、重複キー、試行上限、nullableな同意・期限・取消、停止 |
| `EngineExecution` | 厳密なエンジン版・commit・公開入口・版付き能力証拠・入力／応答指紋・実行結果・診断参照 |
| `TranslationVariant` | 元区間に結び付く、言語別・版付き結果または明示的失敗 |
| `TranslationResult` | 要求全体の終端、結果一覧、エンジン証跡、失敗理由 |
| `DeliveryArtifact` | 成果物、宛先、明示許可、配送結果と不確実状態 |
| `TranslationEvent` | 要求単位で単調増加するイベント番号と状態通知 |

HTTP境界は [`contracts/openapi-v1.json`](../contracts/openapi-v1.json) に定義し、イベントはSSEと順序付き再取得を規定する。スキーマは秘密値を記録しない。媒体内容は原則として参照で結び、監査記録へ本文や音声を複製しない。言語タグはBCP 47形式。`local_only` と `remote_with_consent` を分け、遠隔処理は許可IDなしでは要求として成立しない。

成功状態の訳文にはエンジン実行IDが必要。`UNSUPPORTED`／`SUSPEND`／`FAILED` 等の訳文欄は `null` とし、`COMPLETED` は少なくとも一つの訳結果とエンジン実行証跡を要求する。外部配信は別記録で、`DELIVERED` は配信許可と完了時刻、`DELIVERY_UNKNOWN` は許可と状態不確実性を残す。

JSON Schemaで強制するのはレコード形状と局所条件まで。Rustには同一session/keyの完全一致要求を排他予約する`RequestIdempotencyStore`とプロセス内参照実装がある。同じキーで本文、要求版、attemptが変わればConflictとし、完全一致再送はReplayとして再実行しない。これは永続性や結果取得を提供せず、API/サービス実行経路にも未結線である。ID参照の存在、永続排他、イベント追記、再起動後の復旧、実際の配信停止は全体QCで別途保証しなければならない。

## 4. 入力から出力への正規境界

```text
許可済み媒体入力
  → SourceAsset / SourceSpan（抽出・ASR・OCR・位置／話者／時刻）
  → TranslationSession / TranslationRequest
  → MINIDORA公開入力機 → 本体 → 出力機
  → EngineExecution / TranslationVariant / TranslationResult
  → 許可を確認した DeliveryArtifact
```

Konjacの前処理は原文を観測・抽出し、非意味の話者・時刻・位置情報を付ける。訳文の意味判断・採否を独自に行わない。出力表示、保存、共有、外部送信は異なる操作で、各々の許可を必要とする。

部分更新は要求ID・入力版・区間ID・訳先言語・増加する結果版で相関する。古いイベントで確定結果を巻き戻さない。要求の再試行は入力版と試行を明示し、失敗・キャンセルを黙って成功へ変えない。Rustの`TranslationEventStream`はメモリ内cursorとして要求単位のページ間sequence連続性、時刻順、初回Accepted、終端後の配送順を検査する。イベント取得・結果へのmerge・サービス結線、永続状態機械、checkpoint復旧は未実装。

## 5. 確認・受入

現行JSON/OpenAPI射影、Python能力ゲート、Rust論理境界を検査するには、リポジトリ直下で実行する。

```sh
python -m pip install -r requirements-dev.lock
python -m unittest discover -s tests -v
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Python 25件（JSON Schema/OpenAPI 21件、能力ゲート4件）は射影形状、BCP 47タグ形式、ConsentGrant/TranslationSession/participantのnullable項目・非空同意対象・remote session同意・revision上限・参照重複・`und`、TranslationRequestのnullable項目・整数幅・Unicodeキー長・参照重複、未対応結果の無訳文条件、TranslationEvent nullable参照・u64 sequence上限・event typeごとのvariant/result/artifact参照、SourceAssetの参照/同意/metadata条件、SourceSpanのtagged payload/position/null条件、EngineExecutionの能力証拠/fingerprint/終端対応、TranslationResultのnullable fieldとstatus別完了・失敗条件、TranslationVariantの暫定／確定状態条件、DeliveryArtifactのpayload/宛先/配置参照と配送状態/同意/時刻条件、能力ゲートを検査する。Rust 40件は論理recordのwire version codec、BCP 47構文・正規化、不正言語タグ、同意の主体/対象/用途/期限/撤回と重複grant ID拒否、ConsentGrant主体ID/ConsentRevocation実行者・理由コード文字数、SourceAssetの同意/fingerprint/metadata/参照長、SourceSpanのpayload/position/観測元、EngineExecutionの能力証拠/版一致/fingerprint/終端対応、TranslationResultの進行・終端状態とnullable evidence/失敗コード境界、TranslationVariantの暫定状態と確定実行証跡、DeliveryArtifactのpayload/宛先参照・配置・状態と配送同意/時刻、Deliver scopeとVariant・SourceSpan・versioned capability permit・成功EngineExecutionを一致させる配送preflight、session-scoped冪等claimの完全一致/Conflict/並行予約、ページ間event cursorのsequence・初回Accepted・時刻・終端/配送順、厳密版能力ゲート、能力次元と許可集合の完全一致、アクティブsession/request/input各版、SourceAsset/Span／実行／全訳先結果参照、request時間窓、nullable同意/期限/取消とrevision/attempt/key境界、exchange version変換と不正Envelope拒否、未対応・期限・取消時の実行前停止を検査する。両者とも限定した契約QCであり、IANA registry有効性、エンジン対応、実配信、永続冪等台帳/event log、再起動復旧、射影全同値、認証、永続同意、MINIDORA実行・実訳、実行中停止、サービス統合、対象端末QAの証拠ではない。

実訳受入をPASSへ上げるには、新たに許可されたMINIDORA版の固定commit、公開API、対象言語ペア、文脈、停止／取消の実測、全件証跡が必要。別エンジンの出力や作用注入例で代替しない。
