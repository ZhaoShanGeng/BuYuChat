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

  it("preserves markdown images and hardens image loading attributes", () => {
    const html = renderRichText("![demo](https://example.com/demo.png)");

    expect(html).toContain("<img");
    expect(html).toContain('src="https://example.com/demo.png"');
    expect(html).toContain('loading="lazy"');
    expect(html).toContain('referrerpolicy="no-referrer"');
  });

  it("renders highlighted code blocks with copy metadata", () => {
    const html = renderRichText("```ts\nconst answer = 42;\n```");

    expect(html).toContain("code-block-wrapper");
    expect(html).toContain("code-copy-btn");
    expect(html).toContain("language-ts");
    expect(html).toContain("hljs");
    expect(html).toContain("answer");
  });

  it("renders katex math blocks", () => {
    const html = renderRichText("当 $a^2+b^2=c^2$，则：\n\n$$E = mc^2$$");

    expect(html).toContain("katex");
    expect(html).toContain("katex-display");
    expect(html).toContain("math");
  });

  it("renders katex with tex delimiters", () => {
    const html = renderRichText(
      "这是行内公式：\\(a^2+b^2=c^2\\)\n\n\\[\n\\frac{\\sqrt{a^2+b^2}}{1+\\frac{1}{x}}\n\\]"
    );

    expect(html).toContain("katex");
    expect(html).toContain("katex-display");
    expect(html).toContain("sqrt");
    expect(html).not.toContain("\\(");
    expect(html).not.toContain("\\[");
  });
});
