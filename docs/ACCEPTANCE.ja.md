# こんにゃく翻訳 — 機能・製品受入規定

- 初版：2026-11-04
- 改訂：2026-11-09。Rust共通論理コア、JSON Schema／OpenAPI検査21件、Python能力ゲート検査4件、Rust境界検査39件を実装。ConsentGrant/TranslationSession/participantのnullable項目・非空同意対象・remote session同意・u64 revision・参照重複条件、TranslationRequestのnullable項目・整数幅・Unicodeキー長・参照重複条件、およびTranslationEvent、SourceAsset、SourceSpan、EngineExecution、TranslationResult、TranslationVariant、DeliveryArtifactの射影条件をRust側の局所検証に整合。DeliveryPermitは確定Variant、原文範囲、成功実行と同意を一体で再検査する。製品ランタイムと全経路は未実装。
- 判定語：`PASS`＝条件・証拠が成立、`FAIL`＝不成立、`NOT RUN`＝未実施、`BLOCKED`＝事前依存が不成立、`SUSPEND`＝仕様上判断を停止。単純な「テストがある」を実証としない。

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

## 6. 受入ID別の段階状態

各IDを「仕様定義・実装・結線・QC・環境別QA・Owner最終受入」に分ける。`PASS` はその列の記載範囲だけを指す。契約テストのPASSを製品機能のPASSへ昇格しない。

| ID | 仕様定義 | 実装 | 結線 | QC | QA | Owner最終受入 |
|---|---|---|---|---|---|---|
| A01 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| A02 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| A03 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| A04 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| A05 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| A06 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| A07 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| U01 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U02 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U03 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U04 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U05 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U06 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U07 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U08 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U09 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U10 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |
| U11 | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT RUN |
| U12 | PASS | NOT RUN | BLOCKED | NOT RUN | NOT RUN | NOT RUN |

`A02` と実訳に依存する `A06`／`U01`〜`U12` の結線状態は、凍結 `MINIDORA-BETA 0.6.0b1` の公開翻訳契約を確認できないため `BLOCKED`。この判定は外側の基盤施工を阻止しない。

2026-11-09に [共通契約スキーマ](../contracts/konjac-v1.schema.json)のDraft 2020-12適合、11レコード種の例、明示的遠隔同意、確定訳のエンジン証跡、未対応結果の無訳文、外部配送許可、未知フィールド拒否、ConsentGrant/TranslationSession/participantのnullable項目・非空同意対象・remote同意・revision上限・参照重複、TranslationRequestのnullable項目・整数境界・参照重複条件、TranslationEvent参照要件、SourceAsset参照・同意・metadata条件、SourceSpan tagged payload/positionとnullable field条件、EngineExecution capability証拠/fingerprint/終端対応条件、TranslationResultの進行中／終端条件とnullable evidence、TranslationVariant暫定／確定状態条件、DeliveryArtifact参照・状態・同意・配送時刻条件、[OpenAPI](../contracts/openapi-v1.json)の3.1適合と参照解決、およびMINIDORA版能力プロファイルの閉鎖動作を検査し21件PASSした。これはA05/A06の契約形状QCと依存ゲートQCだけを部分確認するもので、上表の全体実装・結線・QC・QA・最終受入状態は変えない。

製品アプリと全体経路がない現時点での総合状態は **製品受入 `NOT RUN`**、MINIDORA実訳接続は **`BLOCKED`**。RTranslatorを含む対象別QA、品質・性能・安全検収、Ownerによる独立検収は未実施。

2026-11-09に同一作業ツリーでPython 3.12の25件（JSON Schema/OpenAPI形状、BCP 47形式・タグ順序、ConsentGrant/TranslationSession/participantのnullable項目・非空同意対象・remote session同意・u64 revision・参照重複、TranslationRequest nullable項目・整数幅・Unicode冪等キー長・参照重複条件、TranslationEvent種別ごとのvariant/result/artifact参照、SourceAsset参照・同意・metadata条件、SourceSpan payload/position tagとnullable条件、EngineExecution能力証拠/fingerprint/終端対応条件、TranslationResultのstatus別完了時刻・失敗理由・nullable field条件、TranslationVariant暫定／確定の実行証跡、DeliveryArtifact参照・状態・同意・配送時刻条件、unsupported結果の無訳文条件、監査済み能力ゲート）とRust 1.99.1の39件（論理record、BCP 47構文と正規化、同意範囲/期限/撤回と重複grant ID拒否、SourceAssetの同意/fingerprint/metadata/参照長、SourceSpan tagged payload/position/観測元、EngineExecutionの版付きcapability/fingerprint/状態写像/長さ、TranslationRequest nullable項目・u64 revision/u32 attempt・Unicodeキー長・参照重複境界、TranslationResultの進行・終端状態とnullable evidence/失敗コード境界、TranslationVariant暫定訳・確定訳の実行証跡、DeliveryArtifactのpayload/宛先参照・配置・状態と配送同意/時刻、Deliver scopeとVariant・SourceSpan・能力permit・成功EngineExecutionの一致を確認するdelivery preflight、媒体取得/local・remote処理同意、session-scoped冪等keyの初回予約・完全一致再送・本文/版競合・並行排他、イベントページ間のsequence連続性・終端状態、版付き能力ゲート、言語ペア/文脈/モード別許可、アクティブセッションとsession/request/input revision、SourceAsset/Span/実行/全訳先結果参照、要求時間窓、拒否・期限・取消時の実行前停止、配送不変条件）を実行し、全件PASS。Rust `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` もPASS。いずれも局所的な契約QCであり、IANA registry有効性、エンジン対応、実配信、永続イベント/冪等台帳・再起動復旧、製品経路、認証サービス、実行中取消伝播、同意の永続運用、MINIDORA呼出し、A/U全体受入は検証していない。

## 7. E01–E06 環境受入

環境の存在・仕様定義と、その環境での製品適合を分ける。実機確認はQAと独立して記録し、未確認個体を全端末保証へ拡張しない。

| ID | 対象 | 要求定義 | 実装 | 結線 | QC | 環境別QA | 実機確認 | Owner受入 |
|---|---|---|---|---|---|---|---|---|
| E01 | Windows | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |
| E02 | Linux | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |
| E03 | macOS | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |
| E04 | Android | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |
| E05 | iOS | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |
| E06 | Web | PASS | NOT RUN | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED | NOT RUN |

## 8. U01–U12 × E01–E06 適用範囲

`IN`＝その環境に同一利用行為を提供する。`COND`＝OS権限、取得API、対応ブラウザ、アプリ接続、媒体権利に依存し、利用可能経路だけを提供する。表は施工範囲であって実装・適合PASSではない。条件を満たさない場合も行を削除せず、根拠付きで不支持を示す。

| U | E01 Windows | E02 Linux | E03 macOS | E04 Android | E05 iOS | E06 Web |
|---|---|---|---|---|---|---|
| U01 テキスト翻訳 | IN | IN | IN | IN | IN | IN |
| U02 双方向会話 | COND: 音声権限 | COND: 音声サーバー | COND: 音声権限 | COND: マイク/Bluetooth | COND: マイク/Bluetooth | COND: ブラウザ音声権限 |
| U03 録音・音声ファイル | COND: 入力/保存権限 | COND: 音声形式 | COND: 入力/保存権限 | COND: ファイル/背景 | COND: ファイル/背景 | COND: ファイル/メモリ上限 |
| U04 デスクトップ音声字幕 | COND: OS音声API | COND: PipeWire等 | COND: OS音声API | COND: OS提供入力のみ | COND: OS提供入力のみ | COND: ブラウザが許可する音声のみ |
| U05 会議・通話 | COND: 接続先/同意 | COND: 接続先/同意 | COND: 接続先/同意 | COND: アプリ/通話制約 | COND: アプリ/通話制約 | COND: ブラウザ/接続先/同意 |
| U06 動画・配信 | COND: 権利/DRM/API | COND: 権利/DRM/API | COND: 権利/DRM/API | COND: 権利/OS/API | COND: 権利/OS/API | COND: 権利/ブラウザ/DRM |
| U07 Webページ翻訳 | COND: 許可済み拡張等 | COND: 許可済み拡張等 | COND: 許可済み拡張等 | COND: 共有/WebView | COND: 共有/WebView | COND: DOM/サイト権限 |
| U08 文書翻訳 | COND: ファイル権限 | COND: ファイル権限 | COND: ファイル権限 | COND: ファイル/容量 | COND: ファイル/容量 | COND: ファイル/容量 |
| U09 画像・カメラ・OCR | COND: カメラ/画面権限 | COND: カメラ/画面権限 | COND: カメラ/画面権限 | COND: カメラ/共有権限 | COND: カメラ/共有権限 | COND: ブラウザ許可 |
| U10 チャット・メール | COND: 承認済み接続 | COND: 承認済み接続 | COND: 承認済み接続 | COND: 承認済み接続 | COND: 承認済み接続 | COND: 承認済み接続 |
| U11 媒体・端末間継続 | IN | IN | IN | IN | IN | IN |
| U12 API・SDK・連携 | COND: 認証/接続 | COND: 認証/接続 | COND: 認証/接続 | COND: アプリ/API | COND: アプリ/API | COND: 認証/ブラウザ |

全72セルの実装、結線、QC、QA、Owner受入は未実施。U04のモバイル経路はOSが明示的に提供する入力だけが対象で、OS制限の回避を要求しない。ブラウザAPIやホストアプリの実可用性は個別条件として検査する。

