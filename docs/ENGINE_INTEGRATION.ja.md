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

## 6. 2026-11-10 実装・公開能力の観測

これは今回のKonjac runtimeと公開MINIDORA資料を照合した時点の状態である。旧節の設計案や過去のcontrolled smokeを、翻訳能力の証明へ読み替えない。

| 対象 | 観測 | Konjacでの扱い |
|---|---|---|
| 選択依存 `Engine Beta 1 0.6.0b1` | MINIDORA-BETAの公開[API guide](https://github.com/gatchimuchio/MINIDORA-BETA/blob/d907eaaa101ad04a29f83584b1ff39995fbb7d71/docs/API_GUIDE.md)は `minidora.engine_beta.ミニドラβ実行機.実行(...)` の一般テキスト入口、入力／Core／出力、同一executor内の重複抑止を説明する。ページは入力言語の品質を保証せず、translation functionや任意言語ペア受入証拠を記載していない。READMEのcontrolled exampleは接続配線試験である。 | 型付き翻訳API・翻訳受入・利用版固定が未成立。Konjacから未接続。 |
| 別成果のRust候補 `0.7.0-alpha.7` | 公開main SHA `d907eaaa101ad04a29f83584b1ff39995fbb7d71` の[`rust/audit/IMPLEMENTATION_STATUS.md`](https://github.com/gatchimuchio/MINIDORA-BETA/blob/d907eaaa101ad04a29f83584b1ff39995fbb7d71/rust/audit/IMPLEMENTATION_STATUS.md)は「unrestricted translation」を未実装／未検証として明記し、一般Q&A・制約なし文章生成も未実装と記録。READMEも`0.7.0-alpha.7`を独立Rust開発実装として扱う。 | `0.6.0b1`との同一成果・後継採用を仮定しない。能力証拠に使わない。 |
| Konjac `0.1.0` | `POST /api/v1/translate`、`GET /api/v1/capabilities`、 loopback UIが存在する。文字・言語タグ・session/input版を受理する。 | APIは翻訳エンジンを呼ばず、`BLOCKED_ENGINE_CAPABILITY`、`variants=[]`、`delivery_status=NOT_REQUESTED`を返す。要求本文を保存・出力せず、非テキスト媒体は`UNSUPPORTED_ADAPTER`で拒否する。 |

公開資料は選択Betaの任意言語翻訳能力を確定していないため、ここでは能力の不存在ではなく`UNVERIFIED`として扱う。入力が一度受理されたことや任意の1文に結果が返ったことだけでは、言語翻訳の意味保持・一般性・品質受入は成立しない。

### 現在のローカルHTTP契約

- `cargo run --locked -- serve` はloopback IPだけにbindし、UIとJSON入口を同一originで提供する。遠隔利用・認証付きSDK・本番APIではない。
- `TranslationSubmission`は`contract_version`, `session_id`, `input_version`, `source_kind`, `source_text`, `source_language_hint`, `target_languages`を持つ。対象言語は1〜16件、原文は128 KiBまで。契約versionと入力版を検証する。
- `text`以外のsource kindは取得せず、`UNSUPPORTED_ADAPTER`／`NOT_CALLED`を返す。音声・カメラ・画面・ファイルの権限も要求しない。
- `text`はフォームからローカルmemory上の1要求として受理する。選択版に紐づいた実engine executorがないため意味処理は行わない。`BLOCKED_ENGINE_CAPABILITY`とengine証拠参照、訳先別状態のみを返し、`TranslationVariant`を作らない。
- APIの呼出しはloopback内に限るが、本番の利用者認証やU12の認証ジョブ契約は未実装。UI表示と送信・ファイル保存の区別、永続history、cancel/reconcileも未実装である。
- 環境、version、OS別の実訳・返答・配送・結果保存は未試験。ローカルで返るblocked JSONを`WIRED`または実訳として扱わない。

## 7. 2026-11-10 選択版MINIDORA公開入口の隔離実行

Konjac本体の変更ではなく、選択版の公開実行入口が翻訳を成立させるかをLinux／Python 3.12の一時venvで確認した。対象ソースsnapshotは`gatchimuchio/MINIDORA-BETA` commit `d907eaaa101ad04a29f83584b1ff39995fbb7d71`（`0.6.0b1`）、依存は`SudachiPy==0.6.11`と`SudachiDict-core==20260723`。ソースから作ったwheelのSHA-256は`600a58a7357a4d5bd21d73b9cc3464ab1fb86495b0f01a61f2b866ae09945b33`。このwheelは原配布wheelではない。`REPOSITORY_LOCK.md`が示すとおり原配布wheelはGitHub checkoutに含まれず、文書ローカライズを含むsource buildを原バイナリと同一視しない。

- 公開`ミニドラβ実行機.実行`へ明示完了条件なしで依頼すると、完了条件不足の`ValueError`で停止した。
- `要求状態=("翻訳完了",)`を指定し、日本語→英語（「今日は良い天気です。」）、英語→日本語（“The train leaves at noon.”）、フランス語→日本語（“Où est la gare ?”）を各新規executorで実行。全件`SUSPEND`、`HDS_NO_PRODUCTIVE_ACTION`／`作用不足`となり、出力は診断のみで訳文は採用されなかった。
- 実行は完了したが、翻訳受入は成立していない。制御サンプルの`HDS関数作用`注入も行っていない。これは能力の不存在証明ではなく、この版・公開経路・試験条件での失敗観測である。

従ってKonjacの実engine接続は引き続き`NOT_WIRED`、U01は`BLOCKED_ENGINE_CAPABILITY`とする。選択版に翻訳を成立させる公開・受入済み作用／能力が用意されるまで、Konjac側で別の意味エンジンや成功扱いの変換を追加しない。

## 8. 2026-11-10 MINIDORA-BETA最新公開mainの再確認

再確認時の公開mainは[`a55402b86090109117ffc68bfc6abe85b2c86012`](https://github.com/gatchimuchio/MINIDORA-BETA/commit/a55402b86090109117ffc68bfc6abe85b2c86012)。この時点のRust後継は`0.7.0-alpha.8`で、READMEと実装状況はいずれも独立開発alpha・全要求受入未成立と記す。実装状況は、意味比較を有限の日本語表層markerに限定し、一般文の含意判定・無制限翻訳を未実装／未検証として明示している。翻訳consumerやmodalitiesも実provider・権限・障害・privacy等の証拠待ちである。

前回確認した`d907eaaa101ad04a29f83584b1ff39995fbb7d71`からmainまでの3 commit／19 file差分にPython `src/minidora/`の変更はなく、凍結`0.6.0b1`の公開実行入口は変化していない。再確認時、GitHub Releasesは空で、PyPIの`minidora-notnn` project endpointも`404`だった。これらの配布経路に翻訳対応済みの選択可能版は見つからず、Rust alphaを凍結Betaと同一視したり、Konjacの選択版を無断で切り替えたりしない。

同じalpha 8 snapshotを一時checkout／一時Cargo cacheで通常CLIから実行した。`cargo build --locked --release --bin minidora-rust`は成功。日本語→英語（`Translate to English: 今日は良い天気です。`）と英語→日本語（`日本語へ翻訳: The train leaves at noon.`）の双方がexit code 2、`lifecycle=SUSPEND`、`CAPABILITY_UNAVAILABLE`を返し、描画・訳文・外部作用receiptはなかった。checkout、Cargo cache、target、試験状態は終了時に破棄した。これはalpha候補のCLI負例であり、Konjacとの接続試験や製品受入ではない。

結論は前節の隔離実行と同じく、翻訳成功受入`BLOCKED`、Konjac統合`NOT_WIRED`である。新しい公開・受入済みMINIDORA翻訳能力が出るか、オーナーがエンジン方針を変更するまで、この境界を維持する。

