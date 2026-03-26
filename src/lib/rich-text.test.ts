import { describe, expect, it } from "vitest";
import { renderRichText } from "./rich-text";

describe("renderRichText", () => {
  it("renders markdown formatting", () => {
    const html = renderRichText("## 标题\n\n- a\n- **b**");

    expect(html).toContain("<h2>标题</h2>");
    expect(html).toContain("<ul>");
    expect(html).toContain("<strong>b</strong>");
  });

  it("sanitizes unsafe html and hardens links", () => {
    const html = renderRichText(
      '<script>alert(1)</script>\n\n[OpenAI](https://openai.com "site")'
    );

    expect(html).not.toContain("<script>");
    expect(html).toContain('target="_blank"');
    expect(html).toContain('rel="noreferrer noopener"');
  });

  it("renders highlighted code blocks with copy metadata", () => {
    const html = renderRichText("```ts\nconst answer = 42;\n```");

    expect(html).toContain("code-block-wrapper");
    expect(html).toContain("code-copy-btn");
    expect(html).toContain("language-ts");
    expect(html).toContain("hljs");
    expect(html).toContain("answer");
  });
});
