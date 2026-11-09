# こんにゃく翻訳 — 全体系施工・検収計画

- 改訂：2026-11-09
- 正本：Owner提供の[統合製品仕様](implementation-baseline/01_KONJAC_統合製品実装仕様書_v1_20261109.md)、[全体施工工程表](implementation-baseline/02_KONJAC_全体施工工程表_v1_20261109.md)、[Codex施工指示](implementation-baseline/03_KONJAC_Codex全体系施工指示書_v1_20261109.md)
- 状態：全対象射程を保持する施工台帳。工程・機能IDは追跡索引であり、MVP縮小や機能別完成ゲートではない。
- 判定語：`PASS / FAIL / NOT RUN / BLOCKED / SUSPEND`。各工程・施工項目・受入列を独立して判定する。

## 1. G0–G8 全体工程

| 工程 | 成立条件 | 状態 | 根拠・残件 |
|---|---|---|---|
| G0 現物観測 | HEAD、Git履歴、作業差分、全仕様、依存元、権利と既存試験を現物照合し保全 | PASS | [観測報告](OBSERVATION_REPORT.ja.md)。先行差分は保存し照合済み。 |
| G1 仕様整合 | 製品要求・責務境界・全環境・全利用行為・権利・AICompiler交換条件を双方向追跡 | NOT RUN | 主要文書を更新済みだが全要求追跡と矛盾閉鎖は未完了。 |
| G2 実装契約 | 論理モデル、型、状態、EnginePort、権限、保存、交換、構築/配布案を全W/U/E/Aへ接続 | NOT RUN | Rustで主要型、要求版・span・結果参照整合、言語ペア/文脈/モード別能力許可を実装。永続化・全状態・全索引対応とJSON/OpenAPI射影同等性は未完了。 |
| G3 全数BUILD | W01–W12の全媒体・全環境構成を実装し、ビルド可能な成果物を作る | NOT RUN | 現行JSON契約・Python診断QCは部分準備。製品ランタイム・OSアダプターは未施工。 |
| G4 全線結線 | 全入力から能力境界、結果/停止、表示・保存・配送まで共通契約で結線 | NOT RUN | 実訳経路はMINIDORA公開能力未成立のためBLOCKED。外側構成も未結線。 |
| G5 局所QC | 全構成の型、権限、失敗、版、永続化、再試行、競合を検査 | NOT RUN | Python 17件は契約射影・BCP 47形式・イベント参照要件・能力ゲート、Rust 32件は共通コア境界の限定検査であり、G5受入ではない。 |
| G6 全体QC | 全入力→中核→出力、A01–A07、U01–U12の横断経路・異常系を検査 | NOT RUN | 製品全線が未結線。 |
| G7 環境別QA | E01–E06と該当ブラウザ/ABI/権限/配布条件を仮想・自動検証 | NOT RUN | E01 Windows、E02 Linux、E03 macOS、E04 Android、E05 iOS、E06 WebのQAは全てNOT RUN。実機未確認。 |
| G8 Owner最終判定 | Ownerが施工証拠、実訳、品質、権利、残余リスクを独立検収 | NOT RUN | CodexはOwner受入・公開判断を代行しない。 |

G0–G2を閉じてから全体BUILDへ進む。W行ごとに完成判定を作らず、全体結線の後に局所QC、全体QC、環境別QAを行う。安全・型・ビルド整合の施工中検査は許容する。MINIDORA実訳が未成立でも独立して施工可能な外側経路は続けるが、実訳受入はBLOCKEDのままとする。

## 2. W01–W12 全体系施工索引

| ID | 全体責務 | 現在の証拠・不足 | 状態 |
|---|---|---|---|
| W01 | 共通論理契約・型・版 | Rustで7中核record、BCP 47 well-formedタグ構文、ConsentGrant/Revocation、TranslationResult/Eventと主要不変条件・session/request/input revision、SourceAsset/Span/実行/結果参照、能力許可整合を実装。IANA registry検証、JSON/OpenAPI全属性射影と永続版整合は未完了。 | NOT RUN |
| W02 | 権限・同意・信頼・原文保全・監査 | ConsentRegistryで主体/範囲/対象/期限/撤回を評価し、grant ID重複は拒否。認証済みAuthorizationContextは媒体取得と処理grantを能力照会前に検査し、DeliveryPermitはDeliver scopeを対象artifactに束縛して使用時再検証する。OS権限、永続台帳、一括撤回/競合、暗号化、削除、監査保管、復旧は未実装。 | NOT RUN |
| W03 | MINIDORA公開境界ブリッジ | Python監査済み版プロファイルとRust EnginePort/能力ゲート契約あり。`execute_guarded`は期限・取消を確認し、全照会SUPPORTEDのときだけ単一Portを呼ぶ。許可は原言語・訳先・文脈要否・batch/streamingと厳密版証拠を束縛する。凍結0.6.0b1公開APIへの実行アダプター・実行中取消写像なし。翻訳はBLOCKED。 | BLOCKED |
| W04 | テキスト・Web・文書取得 | 仕様と適用行列のみ。テキストUI、DOM、PDF/Office取得は未実装。 | NOT RUN |
| W05 | 音声取得・ASR・話者/時刻 | Haas-ASR/speech.cppを権利・能力調査。統合なし、マイク/音源/ASR経路なし。 | NOT RUN |
| W06 | 画像・映像・OCR・同期 | 仕様と適用行列のみ。画像/カメラ/字幕/映像経路なし。 | NOT RUN |
| W07 | 表示・TTS・字幕・成果物 | DeliveryArtifactの状態検査と別個の配送grant preflightのみ。UI、TTS、実配送アダプター、SRT/VTT、文書/重畳出力なし。 | NOT RUN |
| W08 | セッション・永続化・復旧・重複制御 | 同一session/keyの完全一致要求を排他予約し、再送はReplay、本文・版・attempt違いはConflictとするtraitとメモリ内参照実装を追加。`TranslationEventStream`はページ間のsequence連続性と終端後の配送順序を検査する。DB/永続台帳、サービス結線、イベント追記/checkpoint復旧、版保存なし。 | NOT RUN |
| W09 | Windows/Linux/macOSクライアント | 3環境を要求に保持。製品UI・OSアダプター・配布ビルドなし。 | NOT RUN |
| W10 | Android/iOSクライアント | 2環境を要求に保持。RTranslator責務境界を選定、コード未移植。 | NOT RUN |
| W11 | Web・API/SDK・許可済み接続 | 仕様・OpenAPI投影あり。サービス、認証、API、Web UI、イベント経路なし。 | NOT RUN |
| W12 | ビルド・検証・配布包絡 | Rust 1.99.1、Cargo workspace/lock、共通coreのテスト/format/clippyを現Linux環境で確認。CI、6環境ターゲット/成果物、配布検査なし。 | NOT RUN |

この表は施工漏れの索引であり、W03がBLOCKEDでも他Wを停止しない。第三者ソース・モデルはライセンスと対象ファイルを確認せず取り込まない。

## 3. E01–E06 環境QA

| 環境 | 要求範囲 | 実装 | 結線 | QA | 実機確認 |
|---|---|---|---|---|---|
| E01 Windows | UI、許可済み音声/画面、ファイル、字幕、配布 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |
| E02 Linux | 音声サーバー、Wayland/X11権限、ファイル、配布 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |
| E03 macOS | OS権限、音声/画面、署名、配布 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |
| E04 Android | SDK/ABI、Bluetooth、共有、背景制約、配布 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |
| E05 iOS | SDK/ABI、共有、背景/権限制約、配布 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |
| E06 Web | ブラウザ×OS、許可API、ファイル、通信 | NOT RUN | NOT RUN | NOT RUN | NOT CONFIRMED |

現行Linuxクラウドは開発ホストの証拠に限る。Linux QAやWindows優先製品の成立、他OS互換を示さない。各環境の条件付きU適用は[受入規定](ACCEPTANCE.ja.md)の72セル表に保持する。

## 4. 技術・依存・交換条件

- 共通コアはRustを第一候補として施工する。Rust 1.99.1で共通コアのテスト・format・clippyを現Linuxで確認した。製品ランタイムのビルドやOSターゲットは未施工。
- Tauri 2はWindows/Linux/macOS/Web共有UIの比較候補であり、採用決定・互換性実証ではない。
- PythonはMINIDORA現物診断と契約QCに限定する。
- MINIDORA凍結0.6.0b1に翻訳公開契約が確認できない。MCR-001を記録したが未提出・未成立。代替翻訳器による成功は禁止。
- AICompiler移管は同一の版付き入力・権限で、観測可能な出力、状態、停止、原文対応、配送効果、監査、資源条件を比較する。実装/バイナリ同一性ではない。実装と比較試験は未実施。
- RTranslator 3.0参照コミット、Haas-ASR、speech.cppは出典調査対象として固定済み。コード・モデル・素材は未移植。

## 5. 受入台帳と検証記録

- 構造受入A01–A07、製品受入U01–U12の定義・段階状態は[受入規定](ACCEPTANCE.ja.md)に保持する。定義PASSは実装PASSではない。
- 全72件のU×E適用行列は施工対象の索引であり、各セルの実装・結線・QC・QA・Owner受入は未実施。
- 2026-11-09にPython 3.12の契約/OpenAPI/BCP 47/イベントwire参照/能力ゲート17件とRust 1.99.1の論理record/BCP 47/同意・権限/配送preflight/並行冪等claim/EnginePort/セッション・入力・期限・取消・結果/ページ間イベントcursor境界32件を現作業ツリーで実行し、双方PASS。Rust clippy（warningをerror化）とformat checkもPASS。これらはG5の全W局所QCではない。
- 受入結果の証跡にはHEAD、依存版、OS/CPU/ABI、入力識別、手順、期待値、観測値、未確認範囲を記録する。
- 全体実装完了、全体品質成立、MINIDORA実訳成立、製品受入、公開許可は異なる終端である。

## 6. 判断保留・停止境界

Owner判断を要する重大な外部契約・ライセンス問題、要求母集団の変更、MINIDORA公開契約/凍結版変更、非公開情報公開、商用契約、外部公開・正式リリースは、対象部分を停止して独立施工可能部分を続ける。現在は実訳API変更要求の提出判断、配布/UI基盤の最終採用、モデル権利、処理場所/保存先が未確定である。

