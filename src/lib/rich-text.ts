import DOMPurify, { type Config } from "dompurify";
import { marked } from "marked";
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
import { markedHighlight } from "marked-highlight";

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

function normalizeLanguage(lang: string | undefined): string {
  const normalized = (lang ?? "").trim().toLowerCase();
  if (!normalized) {
    return "text";
  }

  if (hljs.getLanguage(normalized)) {
    return normalized;
  }

  return normalized;
}

marked.use({
  gfm: true,
  breaks: true
});

marked.use(
  markedHighlight({
    langPrefix: "hljs language-",
    highlight(code, lang) {
      if (lang && hljs.getLanguage(lang)) {
        return hljs.highlight(code, { language: lang }).value;
      }

      return hljs.highlightAuto(code).value;
    }
  })
);

marked.use({
  renderer: {
    code(token) {
      const language = normalizeLanguage(token.lang);
      const languageLabel = escapeHtml(language === "text" ? "code" : language);
      const encodedCode = encodeURIComponent(token.text);
      const highlighted =
        language !== "text" && hljs.getLanguage(language)
          ? hljs.highlight(token.text, { language }).value
          : hljs.highlightAuto(token.text).value;

      return [
        '<div class="code-block-wrapper">',
        '  <div class="code-block-header">',
        `    <span class="code-lang">${languageLabel}</span>`,
        `    <button class="code-copy-btn" data-code="${encodedCode}" type="button">复制</button>`,
        "  </div>",
        `  <pre><code class="hljs language-${escapeHtml(language)}">${highlighted}</code></pre>`,
        "</div>"
      ].join("");
    }
  }
});

const SANITIZE_OPTIONS: Config = {
  USE_PROFILES: { html: true },
  FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "form"],
  ADD_ATTR: ["data-code"]
};

export function renderRichText(content: string | null | undefined): string {
  if (!content) {
    return "";
  }

  const dirtyHtml = marked.parse(content) as string;
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

  return template.innerHTML;
}
