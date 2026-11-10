"use strict";

const CONTRACT_VERSION = "konjac.translation.v1";
const ACCEPTED_TRANSLATION_STATUS = "TRANSLATION_ACCEPTED";
const LANGUAGE_SETTINGS_KEY = "konjac.translation.language-settings.v1";
const SESSION_ID = `browser-${globalThis.crypto?.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`}`;
const MAX_TARGETS = 16;
let inputVersion = 1;
const targetLanguages = new Set(["en"]);
const form = document.querySelector("#translation-form");
const sourceText = document.querySelector("#source-text");
const copySourceButton = document.querySelector("#copy-source");
const clearSourceButton = document.querySelector("#clear-source");
const sourceActionStatus = document.querySelector("#source-action-status");
const sourceLanguage = document.querySelector("#source-language");
const targetInput = document.querySelector("#target-language");
const targetList = document.querySelector("#target-list");
const resultContent = document.querySelector("#result-content");
const resultStatus = document.querySelector("#result-status");
const submitButton = document.querySelector("#submit-translation");
const rememberLanguageSettings = document.querySelector("#remember-language-settings");
const clearLanguageSettings = document.querySelector("#clear-language-settings");
const languageSettingsStatus = document.querySelector("#language-settings-status");

function validTag(tag) {
  if (tag.length === 0 || tag.length > 63) return false;
  const parts = tag.split("-");
  return parts.length > 0
    && parts[0].length >= 1 && parts[0].length <= 8 && /^[A-Za-z]+$/.test(parts[0])
    && parts.slice(1).every((part) => part.length >= 1 && part.length <= 8 && /^[A-Za-z0-9]+$/.test(part));
}

function validLanguageSettings(settings) {
  if (settings?.version !== 1 || !Array.isArray(settings.target_languages)) return false;
  const keys = Object.keys(settings).sort();
  if (keys.join(",") !== "source_language_hint,target_languages,version") return false;
  if (settings.target_languages.length > MAX_TARGETS) return false;
  if (!settings.target_languages.every((tag) => typeof tag === "string" && validTag(tag))) return false;
  const normalizedTargets = settings.target_languages.map((tag) => tag.toLowerCase());
  if (new Set(normalizedTargets).size !== normalizedTargets.length) return false;
  return settings.source_language_hint === null
    || (typeof settings.source_language_hint === "string" && validTag(settings.source_language_hint));
}

function loadLanguageSettings() {
  try {
    const stored = localStorage.getItem(LANGUAGE_SETTINGS_KEY);
    if (stored === null) {
      clearLanguageSettings.disabled = true;
      languageSettingsStatus.textContent = "設定は保存されていません。原文や訳文は保存対象外です。";
      return;
    }
    clearLanguageSettings.disabled = false;
    const settings = JSON.parse(stored);
    if (!validLanguageSettings(settings)) {
      languageSettingsStatus.textContent = "保存設定を読み取れません。削除して言語設定をやり直してください。";
      return;
    }
    targetLanguages.clear();
    for (const tag of settings.target_languages) targetLanguages.add(tag);
    sourceLanguage.value = settings.source_language_hint || "";
    rememberLanguageSettings.checked = true;
    languageSettingsStatus.textContent = "保存した言語設定を読み込みました。原文や訳文は保存しません。";
  } catch {
    languageSettingsStatus.textContent = "このブラウザーの保存設定にアクセスできません。";
  }
}

function saveLanguageSettings() {
  if (!rememberLanguageSettings.checked) return;
  const sourceHint = sourceLanguage.value.trim();
  const sourceHintIsValid = sourceHint === "" || validTag(sourceHint);
  const settings = {
    version: 1,
    source_language_hint: sourceHintIsValid ? (sourceHint || null) : null,
    target_languages: [...targetLanguages],
  };
  try {
    localStorage.setItem(LANGUAGE_SETTINGS_KEY, JSON.stringify(settings));
    clearLanguageSettings.disabled = false;
    languageSettingsStatus.textContent = sourceHintIsValid
      ? "言語設定をこのブラウザーに保存しました。原文や訳文は保存しません。"
      : "訳先言語を保存しました。形式が正しくない原言語ヒントは保存していません。";
  } catch {
    languageSettingsStatus.textContent = "設定を保存できませんでした。原文や訳文は保存していません。";
  }
}

function deleteLanguageSettings() {
  try {
    localStorage.removeItem(LANGUAGE_SETTINGS_KEY);
    rememberLanguageSettings.checked = false;
    clearLanguageSettings.disabled = true;
    languageSettingsStatus.textContent = "保存済み言語設定を削除しました。現在の入力は変更していません。";
    return true;
  } catch {
    languageSettingsStatus.textContent = "保存済み設定を削除できませんでした。";
    return false;
  }
}

function renderTargets() {
  targetList.replaceChildren();
  for (const tag of targetLanguages) {
    const chip = document.createElement("span");
    chip.className = "target-chip";
    const text = document.createElement("span");
    text.textContent = tag;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.setAttribute("aria-label", `${tag} を削除`);
    remove.textContent = "×";
    remove.addEventListener("click", () => {
      targetLanguages.delete(tag);
      inputVersion += 1;
      renderTargets();
      saveLanguageSettings();
    });
    chip.append(text, remove);
    targetList.append(chip);
  }
}

document.querySelector("#add-target").addEventListener("click", () => {
  const tag = targetInput.value.trim();
  if (!validTag(tag)) {
    targetInput.setCustomValidity("言語タグを確認してください。例: en, fr, zh-Hant");
    targetInput.reportValidity();
    return;
  }
  targetInput.setCustomValidity("");
  if (targetLanguages.size >= MAX_TARGETS) {
    targetInput.setCustomValidity("訳先は最大16言語です。");
    targetInput.reportValidity();
    return;
  }
  if ([...targetLanguages].some((existing) => existing.toLowerCase() === tag.toLowerCase())) {
    targetInput.setCustomValidity("この言語はすでに追加されています。");
    targetInput.reportValidity();
    return;
  }
  targetLanguages.add(tag);
  inputVersion += 1;
  targetInput.value = "";
  renderTargets();
  saveLanguageSettings();
  targetInput.focus();
});

rememberLanguageSettings.addEventListener("change", () => {
  if (rememberLanguageSettings.checked) saveLanguageSettings();
  else if (!deleteLanguageSettings()) rememberLanguageSettings.checked = true;
});

clearLanguageSettings.addEventListener("click", () => {
  if (!deleteLanguageSettings()) rememberLanguageSettings.checked = true;
});

sourceLanguage.addEventListener("input", () => { inputVersion += 1; });
sourceLanguage.addEventListener("change", saveLanguageSettings);

function updateSourceControls() {
  const bytes = new TextEncoder().encode(sourceText.value).length;
  document.querySelector("#character-count").textContent = `${bytes} bytes`;
  sourceText.setCustomValidity(bytes > 131072 ? "原文は128 KiB以内にしてください。" : "");
  copySourceButton.disabled = sourceText.value.length === 0;
  clearSourceButton.disabled = sourceText.value.length === 0;
  sourceActionStatus.textContent = "";
}

copySourceButton.addEventListener("click", async () => {
  if (sourceText.value.length === 0) return;
  try {
    if (!navigator.clipboard?.writeText) throw new Error("Clipboard API unavailable");
    await navigator.clipboard.writeText(sourceText.value);
    sourceActionStatus.textContent = "原文をクリップボードにコピーしました。";
  } catch {
    sourceActionStatus.textContent = "コピーできませんでした。ブラウザーの権限を確認してください。";
  }
});

clearSourceButton.addEventListener("click", () => {
  if (sourceText.value.length === 0) return;
  sourceText.value = "";
  inputVersion += 1;
  updateSourceControls();
  sourceActionStatus.textContent = "原文を画面から消去しました。";
  sourceText.focus();
});

targetInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    document.querySelector("#add-target").click();
  }
});

sourceText.addEventListener("input", () => {
  inputVersion += 1;
  updateSourceControls();
});

function setResultError(message) {
  const card = document.createElement("div");
  card.className = "outcome";
  const heading = document.createElement("h3");
  heading.textContent = "要求を処理できませんでした";
  const detail = document.createElement("p");
  detail.textContent = message;
  card.append(heading, detail);
  resultContent.replaceChildren(card);
  resultStatus.className = "pill pill-unsupported";
  resultStatus.textContent = "エラー";
}

function showResponse(response) {
  const outcome = document.createElement("div");
  outcome.className = "outcome";
  const heading = document.createElement("h3");
  heading.textContent = response.status === "BLOCKED_ENGINE_CAPABILITY"
    ? "エンジン未対応のため、訳文は生成されませんでした"
    : "この入力方法にはまだ対応していません";
  const detail = document.createElement("p");
  detail.textContent = response.reason;
  outcome.append(heading, detail);

  const targets = document.createElement("div");
  targets.className = "target-outcomes";
  for (const result of response.target_results) {
    const row = document.createElement("div");
    row.className = "target-outcome";
    const language = document.createElement("strong");
    language.textContent = result.target_language;
    const status = document.createElement("span");
    status.textContent = result.status;
    row.append(language, status);
    targets.append(row);
  }
  const empty = document.createElement("p");
  empty.className = "muted result-explanation";
  empty.textContent = response.variants.length === 0 ? "訳文はありません。原文はこの画面内に残っています。" : "";
  outcome.append(targets, empty);
  resultContent.replaceChildren(outcome);
  resultStatus.className = "pill pill-blocked";
  resultStatus.textContent = response.status;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const sourceHint = sourceLanguage.value.trim();
  if (sourceHint && !validTag(sourceHint)) {
    sourceLanguage.setCustomValidity("言語タグを確認してください。例: ja, en, zh-Hant");
    sourceLanguage.reportValidity();
    return;
  }
  sourceLanguage.setCustomValidity("");
  if (!form.reportValidity()) return;
  const targets = [...targetLanguages];
  if (targets.length === 0) {
    targetInput.setCustomValidity("訳先を1つ以上追加してください。");
    targetInput.reportValidity();
    return;
  }
  targetInput.setCustomValidity("");
  submitButton.disabled = true;
  submitButton.querySelector("span:first-child").textContent = "状態を確認中…";
  resultStatus.className = "pill pill-muted";
  resultStatus.textContent = "処理中";
  try {
    // Source text stays in the browser unless the local API explicitly accepts translation.
    const modes = await loadModes();
    if (!modes) {
      setResultError("対応状況を取得できなかったため、原文は送信していません。接続を確認してください。");
      return;
    }
    const textMode = modes.find((mode) => mode.requirement_id === "U01");
    if (textMode?.current_status !== ACCEPTED_TRANSLATION_STATUS) {
      const reportedStatus = textMode?.current_status || "NOT_REPORTED";
      showResponse({
        status: "BLOCKED_ENGINE_CAPABILITY",
        reason: `翻訳能力は受入済みではありません（${reportedStatus}）。原文はこのブラウザー内に残り、ローカルサービスへ送信していません。`,
        target_results: targets.map((target_language) => ({
          target_language,
          status: "BLOCKED_ENGINE_CAPABILITY",
        })),
        variants: [],
      });
      return;
    }
    const response = await fetch("/api/v1/translate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      cache: "no-store",
      body: JSON.stringify({
        contract_version: CONTRACT_VERSION,
        session_id: SESSION_ID,
        input_version: inputVersion,
        source_kind: "text",
        source_text: sourceText.value,
        source_language_hint: sourceHint || null,
        target_languages: targets,
      }),
    });
    const body = await response.json();
    if (!response.ok) {
      throw new Error(body?.error?.message || `HTTP ${response.status}`);
    }
    showResponse(body);
  } catch (error) {
    setResultError(error instanceof Error ? error.message : "ローカルアプリへ接続できませんでした。");
  } finally {
    submitButton.disabled = false;
    submitButton.querySelector("span:first-child").textContent = "翻訳の可否を確認";
  }
});

function renderModes(modes) {
  const list = document.querySelector("#mode-list");
  list.replaceChildren();
  for (const mode of modes) {
    const card = document.createElement("article");
    card.className = "mode-card";
    const heading = document.createElement("h3");
    const code = document.createElement("span");
    code.className = "mode-code";
    code.textContent = mode.requirement_id;
    heading.append(code, document.createTextNode(mode.experience));
    const pill = document.createElement("span");
    const statusClass = mode.current_status.includes("BLOCKED")
      ? "pill-blocked"
      : mode.current_status.includes("PARTIAL") ? "pill-partial" : "pill-unsupported";
    pill.className = `pill ${statusClass}`;
    pill.textContent = mode.current_status;
    const detail = document.createElement("p");
    detail.className = "mode-detail";
    detail.textContent = `入力: ${mode.input_adapter} · Engine: ${mode.engine}`;
    card.append(heading, pill, detail);
    list.append(card);
  }
}

async function loadModes() {
  try {
    const response = await fetch("/api/v1/capabilities", { cache: "no-store" });
    if (!response.ok) throw new Error("HTTP error");
    const modes = await response.json();
    if (!Array.isArray(modes)) throw new Error("Invalid capability response");
    renderModes(modes);
    return modes;
  } catch {
    document.querySelector("#mode-list").textContent = "対応状況を取得できませんでした。";
    return null;
  }
}

loadLanguageSettings();
renderTargets();
updateSourceControls();
loadModes();
