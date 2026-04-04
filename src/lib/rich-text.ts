import DOMPurify, { type Config } from "dompurify";
import MarkdownIt from "markdown-it";
import markdownItHighlightjs from "markdown-it-highlightjs";
import markdownItKatex from "./markdown-it-katex";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import cpp from "highlight.js/lib/languages/cpp";
import css from "highlight.js/lib/languages/css";
import diff from "highlight.js/lib/languages/diff";
import go from "highlight.js/lib/languages/go";
import java from "highlight.js/lib/languages/java";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("shell", bash);
hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("c", cpp);
hljs.registerLanguage("css", css);
hljs.registerLanguage("diff", diff);
hljs.registerLanguage("go", go);
hljs.registerLanguage("java", java);
hljs.registerLanguage("json", json);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("md", markdown);
hljs.registerLanguage("python", python);
hljs.registerLanguage("py", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("rs", rust);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("tsx", typescript);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("yaml", yaml);
hljs.registerLanguage("yml", yaml);

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function encodeDataAttribute(value: string): string {
  return escapeHtml(encodeURIComponent(value));
}

function normalizeLanguage(lang: string | undefined): string {
  const normalized = (lang ?? "").trim().toLowerCase();
  if (!normalized) {
    return "text";
  }

  return normalized;
}

function getFenceLanguage(info: string): string {
  const [language = ""] = info.trim().split(/\s+/, 1);
  return language;
}

function resolvePreviewKind(language: string): "html" | "svg" | "markdown" | null {
  if (language === "html") {
    return "html";
  }

  if (language === "svg") {
    return "svg";
  }

  if (language === "markdown" || language === "md") {
    return "markdown";
  }

  return null;
}

function wrapRawHtmlAsFence(content: string): string {
  if (content.includes("```")) {
    return content;
  }

  const trimmed = content.trim();
  if (!trimmed.startsWith("<")) {
    return content;
  }

  if (/^<svg\b[\s\S]*<\/svg>\s*$/i.test(trimmed)) {
    return `\`\`\`svg\n${trimmed}\n\`\`\``;
  }

  if (
    /^<!doctype html\b[\s\S]*$/i.test(trimmed) ||
    /^<html\b[\s\S]*<\/html>\s*$/i.test(trimmed) ||
    /^<([a-z][\w:-]*)\b[^>]*>[\s\S]*<\/\1>\s*$/i.test(trimmed)
  ) {
    return `\`\`\`html\n${trimmed}\n\`\`\``;
  }

  return content;
}

function wrapCodeBlock(renderedCode: string, language: string, rawCode: string): string {
  const languageLabel = escapeHtml(language === "text" ? "code" : language);
  const encodedCode = encodeURIComponent(rawCode);
  const previewKind = resolvePreviewKind(language);

  if (!previewKind) {
    return [
      '<div class="code-block-wrapper not-prose">',
      '  <div class="code-block-header">',
      `    <span class="code-lang">${languageLabel}</span>`,
      `    <button class="code-copy-btn" data-code="${encodedCode}" type="button">复制</button>`,
      "  </div>",
      `  ${renderedCode}`,
      "</div>"
    ].join("\n");
  }

  return [
    `<div class="code-block-wrapper code-block-wrapper--previewable not-prose" data-active-view="code">`,
    '  <div class="code-block-header">',
    `    <span class="code-lang">${languageLabel}</span>`,
    '    <div class="code-block-actions">',
    '      <div class="code-view-switch" role="tablist" aria-label="代码预览切换">',
    '        <button class="code-view-btn is-active" aria-pressed="true" data-view="code" type="button">源码</button>',
    '        <button class="code-view-btn" aria-pressed="false" data-view="preview" type="button">效果</button>',
    "      </div>",
    '      <button class="code-fullscreen-btn" type="button">全屏</button>',
    `      <button class="code-copy-btn" data-code="${encodedCode}" type="button">复制</button>`,
    "    </div>",
    "  </div>",
    '  <div class="code-block-panels">',
    `    <div class="code-block-panel is-active" data-panel="code">${renderedCode}</div>`,
    `    <div class="code-preview-panel" data-panel="preview" data-preview-kind="${previewKind}" data-preview-source="${encodeDataAttribute(rawCode)}" hidden></div>`,
    "  </div>",
    "</div>"
  ].join("\n");
}

const markdownRenderer = new MarkdownIt({
  breaks: true,
  html: true,
  linkify: true
});

markdownRenderer.use(markdownItHighlightjs, {
  auto: true,
  hljs,
  ignoreIllegals: true
});

markdownRenderer.use(markdownItKatex, {
  strict: "ignore",
  throwOnError: false
});

const defaultFenceRenderer = markdownRenderer.renderer.rules.fence;
if (defaultFenceRenderer) {
  markdownRenderer.renderer.rules.fence = (...args: Parameters<typeof defaultFenceRenderer>) => {
    const [tokens, idx, options, env, self] = args;
    const token = tokens[idx];
    const language = normalizeLanguage(getFenceLanguage(token.info));
    const renderedCode = defaultFenceRenderer(tokens, idx, options, env, self).trim();
    return wrapCodeBlock(renderedCode, language, token.content);
  };
}

const defaultCodeBlockRenderer = markdownRenderer.renderer.rules.code_block;
if (defaultCodeBlockRenderer) {
  markdownRenderer.renderer.rules.code_block = (
    ...args: Parameters<typeof defaultCodeBlockRenderer>
  ) => {
    const [tokens, idx, options, env, self] = args;
    const token = tokens[idx];
    const renderedCode = defaultCodeBlockRenderer(tokens, idx, options, env, self).trim();
    return wrapCodeBlock(renderedCode, "text", token.content);
  };
}

const SANITIZE_OPTIONS: Config = {
  USE_PROFILES: { html: true, mathMl: true },
  FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "form"],
  ADD_ATTR: [
    "data-code",
    "data-view",
    "data-panel",
    "data-preview-kind",
    "data-preview-source",
    "data-active-view",
    "loading",
    "referrerpolicy",
    "srcset"
  ],
  ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|tel):|[^a-z]|data:image\/)/i
};

export function renderRichText(content: string | null | undefined): string {
  if (!content) {
    return "";
  }

  const normalizedContent = wrapRawHtmlAsFence(content);
  const dirtyHtml = markdownRenderer.render(normalizedContent);
  const cleanHtml = DOMPurify.sanitize(dirtyHtml, SANITIZE_OPTIONS);

  if (typeof window === "undefined") {
    return cleanHtml;
  }

  const template = window.document.createElement("template");
  template.innerHTML = cleanHtml;

  for (const link of template.content.querySelectorAll("a")) {
    link.setAttribute("target", "_blank");
    link.setAttribute("rel", "noreferrer noopener");
  }

  for (const image of template.content.querySelectorAll("img")) {
    image.setAttribute("loading", "lazy");
    image.setAttribute("referrerpolicy", "no-referrer");
  }

  return template.innerHTML;
}
