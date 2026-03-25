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
});
