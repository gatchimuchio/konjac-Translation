# Konjac Translation 論理データ契約

- 基準：Owner提供の[統合製品仕様](../docs/implementation-baseline/01_KONJAC_%E7%B5%B1%E5%90%88%E8%A3%BD%E5%93%81%E5%AE%9F%E8%A3%85%E4%BB%95%E6%A7%98%E6%9B%B8_v1_20261109.md) S4–S7
- 状態：**論理契約の正本**。Rust型、JSON Schema、OpenAPI、Protobuf、WASM/ABIは本契約を満たす交換可能な射影であり、いずれか一形式へ製品論理を固定しない。Rust初期実装は一部の型と局所検証までで、全体契約・永続化・投影同値を閉じていない。
- 変更：フィールド意味、責任、状態遷移、観測結果に影響する場合は版を上げ、決定と移行規則を追記する。

## 1. 同一性を持つ中核レコード

### `SourceAsset`

取得元を一意に追跡する。媒体種別、取得経路、元IDまたは暗号学的指紋、権利／所有者参照、入力取得許可、原データ参照、時刻・座標の基準、保持方針を持つ。ソース本文や署名付きURLを診断ログへ複製しない。

### `TranslationSession`

一つの翻訳仕事の識別子、単調増加する設定版、状態、元／訳先言語設定、話者と相手、処理場所、同意参照、文脈・用語参照、保持方針を持つ。媒体をまたぐ継続は、参照先と版が一致する場合にのみ設定を継承する。

### `SourceSpan`

`SourceAsset`とセッション版へ結び付く原文または原媒体範囲。話者、原言語候補、音声時刻区間／文書アンカー／ページ座標、ASR/OCR信頼度と観測元を保持する。訂正・再抽出は以前の内容を消さず新しい入力版として表す。

### `TranslationRequest`

一つ以上の`SourceSpan`とその入力版、`session_id + session_revision`、訳先言語集合、文脈・用語版、出力希望、batch/streaming実行モード、処理場所、期限・取消条件、再試行番号を束ねる。要求の検査時にセッションがアクティブで、言語・文脈・処理方針が一致すること、SourceSpanがSourceAssetと正確な入力版に解決することを検証する。要求IDと冪等性キーは異なる要求を誤って統合しない。対象外・入力不備・権限不足はエンジン成功へ流さない。

### `EngineExecution`

MINIDORAの厳密な配布版／commit、公開入口、要求ID、能力照会結果、正規化した入力・応答指紋、開始／終了、観測した終端とKonjac状態への写像、診断参照を記録する。能力照会は原言語候補・訳先・文脈要否・batch/streamingごとに版付きで行い、要求が必要とする照会集合と許可集合を一致させる。未導入・未監査の版で版情報や成功を捏造しない。意味判断・候補採否の所有者はMINIDORAである。

### `TranslationVariant`

一つの要求の訳先言語・元範囲・結果版・暫定／確定状態を束ねる。確定出力は、対応するMINIDORA実行と入力版を遡れる。ユーザー訂正は新しい版で記録し、原文・以前の確定結果を上書きしない。未成立終端に訳文本文を入れない。

### `DeliveryArtifact`

テキスト、音声、字幕、文書、重畳、API等の出力形式、元Variant、配置・時刻、宛先、別個の配送許可、実際の配送状態・時刻、監査IDを持つ。表示許可、保存許可、外部配送許可を同一視しない。

## 2. 補助レコードと対応関係

同意付与／撤回、認識結果、TranslationResult、順序付きTranslationEvent、操作監査は中核レコードを補う。TranslationResultは要求版と終端、Variant/EngineExecution ID集合を結ぶ。TranslationEventは要求単位の単調sequence、result/variant/artifact参照、発生時刻を結ぶ。補助レコードを追加しても上記7種類の意味関係を変えない。レコード間の参照整合、権限の対象・範囲・期限、版の単調性、配送の一意性は型だけでなく実行時に照合する。

論理的な相関キーは`source_asset_id`、`session_id + session_revision`、`span_id + input_revision`、`request_id + request_revision`、`engine_execution_id`、`target_language + result_revision`、`delivery_artifact_id`。複数言語・媒体・端末・順不同イベントでも原文との対応を失わない。

## 3. 状態と所有者

| 責務 | 許容する外部状態 | 状態の所有者・境界 |
|---|---|---|
| 入力 | `CREATED` / `AUTHORIZED` / `CAPTURED` / `EXTRACTED` / `CANCELLED` / `FAILED` | Konjacの取得・権限アダプター |
| 認識 | `PROVISIONAL` / `FINAL` / `REJECTED` | ASR/OCR等の識別済み処理器。認識確定を翻訳採否と混同しない |
| 翻訳 | `PENDING` / `SUPPORTED` / `UNSUPPORTED` / `SUSPEND` / `FAILED` / `TIMED_OUT` / `CANCELLED` / `RESULT_AVAILABLE` | 能力境界を照会し、意味判断・採否はMINIDORAに委譲 |
| 配送 | `PREPARED` / `AUTHORIZED` / `DELIVERED` / `DELIVERY_FAILED` / `DELIVERY_UNKNOWN` / `REVOKED` | Konjac配送アダプターと別個の配送許可 |

`SUPPORTED`は明示的な版付き能力証拠を要求する。未確認は`SUSPEND`等、既知の非対応は`UNSUPPORTED`とする。取消要求の受付だけで停止完了を主張しない。エンジン結果成立、画面表示、保存、外部配送は別の観測事実とする。

## 4. 実装・シリアライズ射影

Rust共通基盤は本論理モデルの第一実装で、[`crates/konjac-core`](../crates/konjac-core)に7中核record、同意grant/revocation、TranslationResult/Event、主要な局所不変条件、セッション・入力版・SourceAsset・spanの参照検査、全入力/訳先対応の境界検証、言語ペア・文脈要否・実行モードを束縛した閉鎖型能力許可を実装した。EngineRunResult検査は入力span、アクティブセッションと版、要求対象、許可集合、実行証跡のエンジン・能力証拠、訳結果の対応を横断照合する。永続化、状態遷移全体、OS権限、観測/監査保管、保存時のsequence採番と原子的追記は未実装であり、G2/W01/W02は未完了。現在の[`konjac-v1.schema.json`](konjac-v1.schema.json)と[`openapi-v1.json`](openapi-v1.json)は検証可能なJSON射影だが、論理モデルやRust型の全項目・状態を完全に表現していない。G2で生成/検査関係と残る欠落を閉じる。JSON Schema/OpenAPI形式を将来のコンパイル基盤やOS SDKへ漏出させない。

現在確認した射影差分：`SourceAsset`の権利・元指紋・時刻基準、`SourceSpan`のpayload形状、`EngineExecution`の能力状態・照会時刻・入力／応答指紋、`TranslationVariant`／`DeliveryArtifact`の状態と配送許可、記録ごとの版・状態対応、撤回・再起動後の復旧・出力消去が残る。要求のsession/request revisionとexecution modeはJSON射影へ追加したが、Rustとの全フィールド／全状態同値は未検証。現行JSONテストPASSは射影の局所検査であり、論理契約全体の適合証明ではない。

## 5. 実装交換・AICompiler移管

Rustと将来のAICompiler実装を、同一の仕様版・入力集合・許可・状態初期値で比較する。比較対象は観測可能な入出力、状態遷移、終端・停止、原文対応、版、配送効果、監査、資源上限。コード／バイナリ一致は要求しない。差分は仕様、実装、環境、計測のどの原因かを分けて記録する。AICompilerによるRustソースの直接コンパイルや等価性は未確認である。
