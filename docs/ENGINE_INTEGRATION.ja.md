# MINIDORA-BETA 接続・データ契約（設計提案）

制定：2026-11-04。**この契約案はKonjac実装側の設計であり、MINIDORA-BETAに既存翻訳APIが存在するという主張ではない。**

## 1. 駆動と責任の不変条件

- 依存先：[MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA)、Engine Beta 1 `0.6.0b1`。実装・実際のAPI・公開境界は同リポの `README.md`、`AGENTS.md`、`PUBLICATION_BOUNDARY.md`、`REPOSITORY_LOCK.md` に従う。
- ミニドラは **入力機 → 本体 → 出力機**を維持。外部からの翻訳処理は公開エントリポイントに限定する。
- **Konjacは翻訳専用アプリケーション。** 経路の発見、権限、型変換、ASR/OCR、文脈・話者・時間軸の引継ぎ、結果レンダリングを担う。
- 言語の意味判断、MINIDORAの最終採否、学習・駆動状態の正本をKonjac側で新設しない。
- `0.6.0b1`の公開サンプルは制御された入力→作用→出力の接続試験。**任意言語ペアの自由翻訳、ストリーミング翻訳、TTS、OCR、ASRの成立証拠ではない。**

## 2. Konjac側で定義する契約案

次は**新規実装予定**のKonjac外部境界。実装後にスキーマ、型、版、負例テストを固定する。

| 型／識別子 | 必須内容と意義 |
|---|---|
| `SourceAsset` | 入力元種別（text/audio/image/video/document/web/chat）、ソースID、所有/許可範囲、参照/整合性情報。生データをむやみに複製しない |
| `TranslationSession` | `session_id`、利用者の言語設定、参加者/話者、同意・権限・処理場所、文脈参照、セッションの版 |
| `SourceSpan` | `span_id`、原文と対象範囲、`start/end`時刻・元ドキュメント位置、話者、元媒体への対応、ASR/OCR由来の不確実性 |
| `TranslationRequest` | `request_id`、`input_version`、元言語候補、訳先言語**集合**、原文への参照、用語・文脈、求める出力形式、期限・停止/キャンセル条件 |
| `EngineExecution` | 使用したMINIDORAの厳密な版、公開エントリポイント、要求と応答のトレース参照、能力判定、終端・エラー・不支持の状態 |
| `TranslationVariant` | `target_language`、訳文本体、対応`span_id`、`result_version`、エンジン根拠、確定可否、ユーザー修正履歴 |
| `DeliveryArtifact` | subtitle/audio/translated-document/web-overlay/screen-text/apiの出力形式、成果物ID、タイムコード/配置、送信先、配送許可、監査参照 |

**型の位置づけ**：これらは「入力・出力・権限・履歴の包装」であり、隠れた意味モデルや第二のMINIDORAではない。

## 3. 接続ポート案

```text
KonjacInputPort: source -> (authorized) SourceAsset + SourceSpan[]
PerceptionPort: media -> source-aligned observations (VAD / ASR / OCR / extraction)
TranslationBridge: TranslationRequest -> MINIDORA public input/core/output -> EngineExecution
DeliveryPort: EngineExecution + TranslationVariant[] -> authorized DeliveryArtifact[]
```

- `TranslationBridge` はBetaの実在する`ミニドラβ実行機`等の公開機能を呼ぶ前に、能力・引数・出力を**現在のコードで確認**する。Konjac向け `translate(text, from, to)` をMINIDORAに実装済みであるかのように表示してはいけない。
- 能力が欠ける、訳文が確定しない、言語ペアが不支持、内部採否が未成立の場合は `UNSUPPORTED` / `SUSPEND` / `FAILED` 等の**外部状態**として露出する。内部`PASS/SUSPEND/FAIL`等の用語と写像する際は定義を照合。
- 既存NN翻訳器を本番で秘密裏に代替することは、MINIDORAを駆動とする製品の実証を壊す。比較実験の際は実行主体・モード・結果を明示し隔離する。

## 4. ストリーミングと版

外部イベントの例（まだ実装されていない）：

```json
{
  "schema_version": "draft-0",
  "session_id": "s-001",
  "request_id": "r-001",
  "input_version": 1,
  "span_id": "u-003",
  "target_language": "en",
  "event": "provisional_translation",
  "revision": 2,
  "speaker_id": "speaker-a",
  "source_time_ms": [1200, 2700],
  "payload": "<candidate text>"
}
```

- 逐次ASRが`暫定`を更新しても、確定訳は古いイベントで巻き戻さない。`source` の確定・`engine` の採否・`delivery` の表示確定は別段階。
- 同じ発話を複数言語へ翻訳する場合は、同じ元`span_id`と個別の訳先・版を持つ。
- 話者変更、時刻ズレ、文書位置ズレ、逆向き会話、結果欠落は明示的に追跡する。
- エンジン側の重複入力抑止は**同一インスタンス内**など公開された成立範囲だけを前提とし、Konjac側に永続的idempotency／配送記録が必要なら別途実装・試験する。

## 5. 実装前に必要な成立証拠

1. Beta公開APIに対し、実際の入力・境界・出力と停止条件を監査する。
2. 元言語・訳先言語・用語・文脈を伴う最小の**実訳**を成功させる。与えられた固定作用の実行と混同しない。
3. 二言語・複数言語・長文・曖昧表現・固有名詞・同音異義・逆翻訳を固定データで検証。
4. エラー／タイムアウト／キャンセル／再試行時の`SUSPEND`・失敗・無配送を検証。
5. Windows実機・Web・Android/iOS等それぞれの入出力境界を個別に確認する。
6. 新APIをMINIDORA側に要する場合は**Betaを直接改造せず**、独立版の提案と非退行検証を別案件として扱う。

以上を満たすまで「MINIDORA翻訳機能完成」は未検証のまま保持する。

