import DOMPurify, { type Config } from "dompurify";
import { marked } from "marked";

marked.use({
  gfm: true,
  breaks: true
});

const SANITIZE_OPTIONS: Config = {
  USE_PROFILES: { html: true },
  FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "form"]
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
