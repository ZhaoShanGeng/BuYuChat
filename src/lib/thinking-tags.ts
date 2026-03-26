export type ThinkingExtractResult = {
  thinking: string | null;
  body: string;
};

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function parseThinkingTagsConfig(raw: string | null | undefined): string[] {
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .map((item) => (typeof item === "string" ? item.trim().toLowerCase() : ""))
      .filter((item, index, items) => item.length > 0 && items.indexOf(item) === index);
  } catch {
    return [];
  }
}

export function serializeThinkingTagsInput(value: string): string | null {
  const tags = value
    .split(",")
    .map((item) => item.trim().toLowerCase())
    .filter((item, index, items) => item.length > 0 && items.indexOf(item) === index);

  return tags.length > 0 ? JSON.stringify(tags) : null;
}

export function extractThinkingTags(
  content: string,
  tags: string[]
): ThinkingExtractResult {
  if (!content || tags.length === 0) {
    return { thinking: null, body: content };
  }

  const pattern = tags.map(escapeRegExp).join("|");
  const regex = new RegExp(`<(${pattern})>([\\s\\S]*?)<\\/\\1>`, "gi");
  const thinkingParts: string[] = [];
  const body = content.replace(regex, (_, __, inner: string) => {
    const value = inner.trim();
    if (value) {
      thinkingParts.push(value);
    }
    return "";
  });

  return {
    thinking: thinkingParts.length > 0 ? thinkingParts.join("\n\n") : null,
    body: body.replace(/\n{3,}/g, "\n\n").trim()
  };
}
