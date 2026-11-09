# 参照ソフトウェア解析・移植境界

- 観測根拠：GitHub公開コード・公式文書。再確認日：2026-11-09。
- 目的：**こんにゃく翻訳の外側に必要なアーキテクチャ**を抽出すること。MINIDORA内部の代替エンジン探索ではない。
- 区別：**既存コードで観測した内容**、**Ownerが定めた外部基盤方針**、**Konjacへ取り込まれた実装**を分ける。

## 1. 比較と採用候補

| 参照元 | 観測できる技術／挙動 | Konjac側で参考にする部分 | 検証を要する制約 |
|---|---|---|---|
| [RTranslator 3.0](https://github.com/niedev/RTranslator/tree/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e) | Android音声サービス、録音・VAD・ASR、既存翻訳器、システムTTS、会話UI、Bluetooth端末間会話、モデル資源管理 | **Owner選定済みのAndroid外部基盤**。端末ライフサイクル、音声取得／停止、会話UI、Bluetooth接続、TTS境界、権限・資源管理 | Android特化。既存翻訳器とモデルはMINIDORA駆動と分離し除外。個別ファイルと依存物は導入時に監査 |
| [Haas-ASR](https://github.com/third-intelligence/Haas-ASR/tree/acc614f68769dcbfae9dba96b48439c4a66295af) | `acc614f68769dcbfae9dba96b48439c4a66295af`。日本語ASRコード／重み、長尺区間分割、話者例、マイクの疑似ライブデモ | 区間・時刻・進捗・暫定／確定イベントの境界だけを参考にする | READMEは公式ASRモデルと重みをApache-2.0と記載。Linux＋NVIDIA GPUを要求。疑似ライブは定期的な全区間再認識のデモで、製品ストリーミングや翻訳ではない。NeMo/Flash Linear Attention等の依存と話者モデルは別監査 |
| [speech.cpp](https://github.com/nyosegawa/speech.cpp/tree/03555100287c118b41d1b475044e5ab1e07baa8c) | `03555100287c118b41d1b475044e5ab1e07baa8c`。C APIでローカルモデルのASR/TTS/VAD/話者処理、要求状態・取消を公開。HTTP/WebSocketサーバーは別実行経路 | C/C++境界、型付き結果・エラー・取消、音声前後処理の交換点 | ソースはMIT。モデルカードによりApache-2.0、MIT＋倫理制約、Gemma Terms、Fish Audio Community、OpenMDW、CC-BY等が異なる。C API自体はモデルを内包せず、選定モデルと配布条件を個別監査 |
| [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA) | 入力機／駆動コア／出力機を備えるβ公開実装、限定受入テストと公開I/O | **唯一の選択済み駆動エンジン**との接続契約 | 任意言語ペア翻訳・実時間翻訳・クロス端末翻訳は未実証 |

なお各参照元は同一種類の翻訳エンジンではない。RTranslatorのASR/翻訳モデル、HaasのASRモデル、speech.cppの音声処理モデルは、MINIDORAの意味処理・採否を代替しない。共通に抽出する対象は**媒体／音声／通信／UIを扱う外側の機構と、翻訳駆動との交換点**。

## 2. RTranslator 3.0 の採用方針と観測範囲

方針：RTranslator 3.0 をAndroid側の外部構成基盤に選定する。これは**基盤の選定**であり、ソースコードの取り込み・翻訳モデルの採用・Android製品の成立を意味しない。2026-11-09に `v3.00` ブランチをcheckoutして得たcommitは `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e`。同時点の `app/build.gradle` は `3.0.0-beta2`、`minSdkVersion 28`、`arm64-v8a` を指定する。従って「3.0」は安定版保証や他OS対応の主張ではない。

| 再利用境界 | 対象と条件 |
|---|---|
| 移植を検討する外部機構 | `VoiceTranslationService.java` のサービス／イベントライフサイクル、`Recorder.java` の音声バッファ／VAD／停止再開、`ConversationService.java` のBluetooth通信境界、`TTSEngine.java` のAndroidシステムTTS境界、会話UIと権限状態、汎用モデル資源の状態表示 |
| 抽出時の分離 | Android UI・権限／OS API・通信・音声入出力を各アダプター内へ閉じる。各アダプターは共有JSON契約を介し、独自の翻訳判断を持たない |
| 再利用しない | `Translator.java`、`DictionaryTranslator.java`、`BergamotTranslator.java`、翻訳言語モデル／辞書・重み、翻訳成功を作る既存パイプライン。ASR/OCR等のモデルや依存物も個別権利・性能監査前は含めない |
| 依存・配布 | RTranslator全体のGradle依存、ネイティブABI、署名設定をそのまま継承しない。必要な依存はパッケージ・版・ライセンス・供給元・モデル／素材条件を別記録し、実機負荷を検証 |

観測箇所：

| 公開コード | 観測した境界 |
|---|---|
| [VoiceTranslationService.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/VoiceTranslationService.java) | フォアグラウンドサービス、入力/出力切替、UIへのイベント通知、音声ライフサイクル |
| [Recorder.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/neural_networks/voice/Recorder.java) | 録音バッファ・VAD・発話境界とマイク停止/再開 |
| [Translator.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/neural_networks/translation/Translator.java) | 部分訳／確定訳のコールバック、言語ペア・出力の情報。既存モデルとの結合は直接採用しない |
| [ConversationService.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/_conversation_mode/_conversation/ConversationService.java) | Bluetooth端末間テキスト受け渡しと会話表示。インターネット通話とは区別 |
| [TTSEngine.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/TTSEngine.java) | Android TTSの再生境界 |
| [LICENSE.txt](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/LICENSE.txt) / [NOTICE.txt](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/NOTICE.txt) | 参照時に確認したApache-2.0本文と通知 |

ルートLICENSEはApache-2.0。導入対象ファイルのヘッダー、LICENSE、NOTICE、同梱素材と各推移依存の条件も個別に照合し、改変ファイルには変更通知を保持する。Apache-2.0は翻訳モデル、音声素材、UI素材、Maven依存を一括して許諾するものではない。音声ログ、Bluetoothの脅威モデル、デバッグ署名、モデル配布は移植前に個別監査する。

この作業では参照ソースをKonjacへコピーしていない。外部基盤選定とコード導入済み状態を混同しない。

## 3. Haas-ASRの具体的な観測箇所

基準：2026-11-09に固定commit `acc614f68769dcbfae9dba96b48439c4a66295af` を取得し、LICENSE、README、推論・長尺・音声区間ソースを確認。

- [demos/README.md](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/README.md)：公式に**非ストリーミングASRを約250msおきに再認識**する疑似ライブデモと説明。`0.7秒`無音または`14秒`上限で区間確定。**非製品向けデモ**と明記。
- [demos/realtime.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/realtime.py)：音声ストリーム、暫定/確定更新、測時。
- [demos/segmenter.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/segmenter.py)：音声区間分割。固定音量閾値の雑音・小声への弱点を確認する。
- [demos/longform.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/longform.py)：長尺音声の分割と時刻付き進捗通知。

Haas-ASRで観測された日本語認識率やGPU速度を、こんにゃく翻訳の全言語翻訳品質やAndroid速度として転用しない。

## 4. speech.cpp の具体的な観測箇所

基準：2026-11-09に固定commit `03555100287c118b41d1b475044e5ab1e07baa8c` を取得し、LICENSE、README、C API、要求・取消ソースを確認。

- [docs/c-api.md](https://github.com/nyosegawa/speech.cpp/blob/03555100287c118b41d1b475044e5ab1e07baa8c/docs/c-api.md)：モデル・要求の生成/破棄、`speech_request_set_audio`、`speech_transcribe`、`speech_synthesize`、型・タイムスタンプ・中断・エラー。
- [docs/server.md](https://github.com/nyosegawa/speech.cpp/blob/03555100287c118b41d1b475044e5ab1e07baa8c/docs/server.md)：`/v1/audio/transcriptions`、`/v1/audio/speech`、`/v1/realtime`などの接続形態。

本書はインターフェースの参考とするだけで、C++/ggml・音声モデルをKonjacへ組み込んでいない。

## 5. ライセンス・導入ゲート

- RTranslatorのソース：Apache-2.0。Haas-ASRのREADMEは当該ASRコードとHaas-ja-v1.0重みをApache-2.0と明記するが、データと推移依存は個別確認。speech.cppのソース：MIT。音声モデル・素材・派生資産にはモデルカード等の別条件がある。
- 現時点で、これらの第三者のソース・モデル・データを本リポへコピーしていない。`NOTICE`には参照元として記録する。
- 取り込み前に、正確な版／出典／ファイル名／変更箇所／ライセンス条項／元NOTICE／再配布義務／動作環境を固定し、出典と採用判断を更新する。
- 参照元の特徴を**取り込めること**と、Konjacに**取り込まれ実装されたこと**を別に記録する。

## 6. 採用状態

**RTranslator 3.0 をAndroid外部基盤として選定した。** 移植単位は責務で絞り、MINIDORAと競合する翻訳器・翻訳モデルを分離する。個別ファイルの移植、モデル・素材採用、Androidビルドは未実施。Windows/Web/iOS/APIはRTranslator採用範囲に含めず、共有契約を利用する別アダプターとして扱う。

