# こんにゃく翻訳 — 機能・製品受入規定

- 初版：2026-11-04
- 初期状態（2026-11-04）：実装・実行試験なし。以下は製品受入条件であり、個別実装の存在だけで全体受入とはしない。
- 判定語：`PASS`＝条件・証拠が成立、`FAIL`＝不成立、`NOT RUN`＝未実施、`BLOCKED`＝事前依存が不成立、`SUSPEND`＝仕様上判断を停止。単純な「テストがある」を実証としない。

> **保証対象の補足（2026-11-09決定）**：A01〜A07/U01〜U12と品質・安全の母集団は保持する。品質保証は固定した仕様・版・入力・仮定・適用環境に対する再現可能な試験（仮想・シミュレーションを含む）と証拠で範囲を明示して判定する。実機での確認は宣言した物理対象の適用確認であり、すべての仮想試験を無効化する一律の前提条件ではない。一方、未観測のOS・端末の個別動作や全機種互換を保証済みと主張しない。詳細は[統合実行規定](../AGENTS.md)と[意思決定記録](DECISIONS.md)。

## 1. 構造に関する必須検査

| ID | 受入条件 |
|---|---|
| A01 | 製品の主語が翻訳体験に固定され、全媒体カテゴリが製品仕様に残っている |
| A02 | MINIDORA-BETAを公開API経由で参照し、内部の入力機→本体→出力機と能力継承を破壊しない |
| A03 | 翻訳採否・学習・意味判断の第二機構がKonjac外側に存在しない |
| A04 | 専用ASR/OCR/TTS・通信・字幕・レイアウト再構成の差替が中核変更なしでできる |
| A05 | データ契約で原文、訳文、言語ペア、話者、時刻、媒体位置、版、処理場所、出典の対応を保つ |
| A06 | MINIDORA能力欠落／未対応／失敗を成功へ昇格しない。代替エンジンの使用は明示実験に隔離 |
| A07 | ライセンス帰属と機密公開境界を守り、第三者モデルの再配布条件を監査する |

## 2. 利用場面の受入行列

| ID | 利用者の操作と観測すべき結果 |
|---|---|
| U01 | 文章→1言語および複数言語へ翻訳、原文対照・コピー・編集・履歴・明確な失敗 |
| U02 | スマホ双方向会話→話者切替・字幕／翻訳音声、発話重複・回り込み・無音を適切に扱う |
| U03 | 録音→時刻付き文字起こし→複数言語へ訳→SRT/VTT/文章等の成果物 |
| U04 | Windows上で許可された音声源→逐次字幕。暫定/確定、遅延表示、キャンセル、再開を検証 |
| U05 | 会議／通話→参加者別、言語別、入力権限、議事録、未対応アプリの説明 |
| U06 | 動画・配信→字幕同期・シーク・途中再生・複数音声・吹替の対応確認 |
| U07 | Web→文書構造／リンク／入力欄／引用の関係を保持。権限外のDOM改変なし |
| U08 | PDF/Office→見出し・表・脚注・画像位置を維持、保持不能箇所を申告 |
| U09 | カメラ／画像→OCR原文と訳文の位置対応、文字誤読・照明変化・縦書き等の処理 |
| U10 | チャット／メール→受信表示・返信案の翻訳、**利用者の許可なく送信しない** |
| U11 | 同一の言語設定、文脈、相手、用語を媒体／端末間で適切に継承・修正できる |
| U12 | API/SDK→認証済みジョブ・イベント・出力・キャンセル・再試行・レート制御 |

## 3. 品質・性能

- **翻訳品質**：意味保持、用語・固有名詞・敬語・文脈・曖昧性・長文、言語ペア別の人手評価と対照サンプルを記録。自動指標だけで「実用」を宣言しない。
- **認識品質**：ASRはWER/CER、話者分離、雑音・距離・重なり、OCRは文字・位置正確性を測る。第三者論文の値を製品値として使わない。
- **遅延**：発話時点→認識暫定→翻訳暫定→確定→表示/TTSの分布（例：中央値・p95）。測定区間と端末を固定し、根拠なしの閾値を決め打ちしない。
- **同期**：字幕の対応範囲と時刻差、語順変化、映像シーク後の復帰、吹替の話速整合。
- **資源**：Windows実機を含むOS別CPU/GPU/メモリ／ディスク／バッテリー、起動、モデルサイズ、温度、使用権限を計測。
- **UX**：翻訳までの操作回数、設定の再入力、誤訳修正と原文復帰、翻訳中断時の復旧、アクセシビリティをシナリオテスト。

性能閾値の設定は比較対象・利用者目標・実測データが揃ってから行う。未設定を合格として扱わない。

## 4. 安全と障害

- 無許可のマイク、カメラ、画面、通話録音、クリップボード読取、外部送信、他アプリへの書込、クラウド処理を負例で拒否。
- ローカル／遠隔の切替表示と同意、保管期間・消去、ログの原文非露出、暗号化、第三者サーバー障害時の停止。
- 入力言語不明、未対応文字、権限取消、途中切断、重複入力、結果到着順の逆転、出力先喪失、モデル非搭載、キャンセル競合を監査。
- `CANCELLED`、`TIMED_OUT`、`UNSUPPORTED`、`SUSPEND`、`DELIVERED`、`DELIVERY_UNKNOWN` の差を保持。キャンセル受付を処理停止の証明としない。
- 原文と訳文の修正履歴を消さず、再生／復旧で再配信や重複送信しない。

## 5. 検証証跡

各検査ケースに最低限、対象版・コミット・入力ファイル/テストデータ識別・OS/CPU/GPU・モデル版・テスト時刻・実行手順・結果・ログ場所・期待値・観測値・未確認点を記録する。

## 6. 2026-11-10 実装基線の局所状態

下記は製品受入のPASSではなく、Rust `0.1.0`の局所コード状態を示す。検証条件・コマンドの結果は施工時点の報告を参照する。

| ID | 現在の実装／観測 | 状態 |
|---|---|---|
| A01 | UI/APIはU01〜U12を能力行列に残し、製品の全媒体scopeを表示 | `CODE_PRESENT`（全媒体の利用成立は未確認） |
| A02 | MINIDORA公開APIへの呼出し・接続版固定はない | `NOT_WIRED` |
| A03 | Konjacに翻訳意味判断・採否の第二engineはない | `CODE_PRESENT`（追加diff範囲） |
| A04 | source kind列挙と状態表示のみ。交換可能なadapter port・ASR/OCR/TTS実物はない | `PARTIAL_CODE_PRESENT` |
| A05 | text session/version/source language/target languageの局所契約あり。原asset hash/ref、話者・時刻・位置等の全媒体相関はない | `PARTIAL_CODE_PRESENT` |
| A06 | engine能力未確認時は訳文0件・配送なしで停止する | `TESTED`（局所境界のみ） |
| A07 | dependency lock・package license metadataを記録。第三者コード・モデル・dataの取り込みなし | `CODE_PRESENT`（製品全体の出荷監査は未成立） |
| U01 | 文字入力、利用者操作の原文コピー／消去、loopback JSON入口からblock理由を表示 | `PARTIAL_CODE_PRESENT` / translation `BLOCKED_ENGINE_CAPABILITY` |
| U02–U10 | 会話・音声・会議・動画・Web・文書・画像・chat adapterなし | `NOT_IMPLEMENTED` |
| U11 | 明示opt-inのブラウザー内言語タグ保存・削除のみ。文脈引継ぎ・媒体／端末間同期なし | `PARTIAL_CODE_PRESENT` |
| U12 | loopback text endpointのみ。利用者認証・SDK・remote jobs・cancel/deliveryなし | `PARTIAL_CODE_PRESENT` |

**製品受入判定：A01〜A07、U01〜U12の要求全体、翻訳品質・性能・アクセシビリティ・全媒体安全・Windows/Web/mobile対象QA・release gateは `NOT RUN` または能力依存により `BLOCKED`。** テストPASSは上表の局所状態だけを裏付け、実訳・接続・製品完成へ拡張しない。

### この施工の検証証跡

- 対象：`konjac-Translation`、Rust `0.1.0`、Linux x86_64、Rust/Cargo `1.99.1`、locked dependency set。Konjac本体の検査ではMINIDORAを呼んでいない。別実行の隔離公開API観測は後述。
- `cargo fmt --all -- --check` — PASS。
- `cargo check --workspace --all-targets --locked` — PASS。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS。
- `cargo test --workspace --locked` — PASS、7 unit tests。FAIL CLOSED、未対応media拒否、訳先検証、loopback HTTP処理、source非反映、cross-origin拒否を含む。
- `cargo build --workspace --locked --release` — PASS。
- `node --check web/app.js` — PASS。Playwright／Chromium (`/usr/bin/chromium`)でローカル画面を開き、U01〜U12の12行、`en`/`fr`の二訳先、BLOCKED表示、訳文なし、browser page errorなしを確認 — PASS。
- 追加loopback HTTP観測：未対応画像入力=`UNSUPPORTED_ADAPTER`かつengine未呼出、cross-origin=`403`、誤Content-Type=`415`、重複訳先=`422` — PASS。
- 最初のPlaywright assertionはバナーと結果欄の同じstatus textをstrict selectorで同時取得し、確認不能となった。結果欄の固有IDへselectorを限定して同じbuildで再実行しPASS。製品コードの変更は不要だった。
- MINIDORA隔離公開API観測：`0.6.0b1`ソースbuildを一時venvに導入し、明示完了条件ありの3言語方向を実行。全件`SUSPEND`／`HDS_NO_PRODUCTIVE_ACTION`で、訳文採用なし。詳細とsource SHAは`docs/ENGINE_INTEGRATION.ja.md` §7を参照。
- MINIDORA別成果候補観測：`0.7.0-alpha.8` commit `a55402b86090109117ffc68bfc6abe85b2c86012`のCLIを隔離buildし、日本語↔英語の2依頼を実行。buildはPASS、双方`SUSPEND`／`CAPABILITY_UNAVAILABLE`で訳文なし。候補版は選択・統合していない。詳細は`docs/ENGINE_INTEGRATION.ja.md` §8。
- ブラウザー送信前preflight：loopback UIでblocked status時は能力GETのみでPOSTなし、入力をフォームに保持、page errorなしをPlaywright／Chromiumで確認。能力GET失敗時も原文をPOSTせずエラー表示。合成preflightで`TRANSLATION_ACCEPTED`を返した制御試験ではPOST経路に進むが、応答はmock blockedであり、実翻訳の証拠ではない。
- U11のブラウザー保存：初期offでは原言語ヒント・訳先言語タグを再読込後も保存しない。明示opt-in後はversion、原言語ヒント、訳先言語タグのみをlocalStorageへ保存し、固有の原文を含めないこと、再読込で設定を復元すること、削除操作でキーを消し再読込後は初期値へ戻ることをPlaywright／Chromiumで確認 — PASS。訳文は未生成であり、文脈・人物・用語の保持、他媒体／端末との同期は未実装。
- U01の原文操作：Playwright／Chromiumで利用者クリック後に限りクリップボードへコピーされること、クリップボードを読み取らないこと、消去操作で画面の原文とbyte counterを初期化すること、コピー・消去ボタンが空入力時に無効であることを確認 — PASS。自動送信・自動保存は行わない。
- U01の送信相関：合成preflightに`TRANSLATION_ACCEPTED`を返す制御試験で、同じページ内の複数送信が同一session IDを使い、原文・原言語ヒント・訳先の編集に応じてinput versionが増えることを確認 — PASS。API応答はmock blockedで、実訳の証拠ではない。再読込でsession ID／版は初期化される。
- `BLOCKED`: 選択MINIDORA版での成功翻訳受入。`NOT RUN`: Konjacからのengine統合、原文・訳文・セッション文脈等の保持／消去、Windows／macOS／Android／iOS/Web配布、実機権限、remote effects、全Uxx品質受入、性能測定、独立QA。

