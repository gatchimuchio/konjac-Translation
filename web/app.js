"use strict";

const CONTRACT_VERSION = "konjac.translation.v1";
const MAX_TARGETS = 16;
const targetLanguages = new Set(["en"]);
const form = document.querySelector("#translation-form");
const sourceText = document.querySelector("#source-text");
const sourceLanguage = document.querySelector("#source-language");
const targetInput = document.querySelector("#target-language");
const targetList = document.querySelector("#target-list");
const resultContent = document.querySelector("#result-content");
const resultStatus = document.querySelector("#result-status");
const submitButton = document.querySelector("#submit-translation");

function validTag(tag) {
  if (tag.length === 0 || tag.length > 63) return false;
  const parts = tag.split("-");
  return parts.length > 0
    && parts[0].length >= 1 && parts[0].length <= 8 && /^[A-Za-z]+$/.test(parts[0])
    && parts.slice(1).every((part) => part.length >= 1 && part.length <= 8 && /^[A-Za-z0-9]+$/.test(part));
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
      renderTargets();
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
  targetInput.value = "";
  renderTargets();
  targetInput.focus();
});

targetInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    document.querySelector("#add-target").click();
  }
});

sourceText.addEventListener("input", () => {
  const bytes = new TextEncoder().encode(sourceText.value).length;
  document.querySelector("#character-count").textContent = `${bytes} bytes`;
  sourceText.setCustomValidity(bytes > 131072 ? "原文は128 KiB以内にしてください。" : "");
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
    const response = await fetch("/api/v1/translate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      cache: "no-store",
      body: JSON.stringify({
        contract_version: CONTRACT_VERSION,
        session_id: `browser-${globalThis.crypto?.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`}`,
        input_version: 1,
        source_kind: "text",
        source_text: sourceText.value,
        source_language_hint: sourceLanguage.value.trim() || null,
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
    pill.className = `pill ${mode.current_status.includes("BLOCKED") ? "pill-blocked" : "pill-unsupported"}`;
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
    renderModes(await response.json());
  } catch {
    document.querySelector("#mode-list").textContent = "対応状況を取得できませんでした。";
  }
}

renderTargets();
loadModes();
