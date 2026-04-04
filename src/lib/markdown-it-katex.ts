import { renderToString, type KatexOptions } from "katex";
import type MarkdownIt from "markdown-it";

type MathDelimiters = {
  canClose: boolean;
  canOpen: boolean;
};

type InlineMathMarker = {
  close: string;
  markup: string;
};

type BlockMathMarker = {
  close: string;
  markup: string;
};

export type MarkdownItKatexOptions = KatexOptions;

function isValidDelimiter(state: any, position: number): MathDelimiters {
  const prevChar = position > 0 ? state.src.charCodeAt(position - 1) : -1;
  const nextChar = position + 1 <= state.posMax ? state.src.charCodeAt(position + 1) : -1;

  let canOpen = true;
  let canClose = true;

  if (
    prevChar === 0x20 ||
    prevChar === 0x09 ||
    (nextChar >= 0x30 && nextChar <= 0x39)
  ) {
    canClose = false;
  }

  if (nextChar === 0x20 || nextChar === 0x09) {
    canOpen = false;
  }

  return { canClose, canOpen };
}

function renderKatex(content: string, options: MarkdownItKatexOptions, displayMode: boolean): string {
  try {
    return renderToString(content, { ...options, displayMode });
  } catch (error) {
    if (options.throwOnError) {
      console.error(error);
    }
    return content;
  }
}

function getInlineMathMarker(state: any): InlineMathMarker | null {
  if (state.src[state.pos] === "$" && state.src[state.pos + 1] !== "$") {
    return {
      close: "$",
      markup: "$"
    };
  }

  if (state.src.slice(state.pos, state.pos + 2) === "\\(") {
    return {
      close: "\\)",
      markup: "\\("
    };
  }

  return null;
}

function getBlockMathMarker(state: any, position: number): BlockMathMarker | null {
  if (state.src.slice(position, position + 2) === "$$") {
    return {
      close: "$$",
      markup: "$$"
    };
  }

  if (state.src.slice(position, position + 2) === "\\[") {
    return {
      close: "\\]",
      markup: "\\["
    };
  }

  return null;
}

function mathInline(state: any, silent: boolean): boolean {
  const marker = getInlineMathMarker(state);
  if (!marker) {
    return false;
  }

  if (marker.markup === "$") {
    const startDelimiter = isValidDelimiter(state, state.pos);
    if (!startDelimiter.canOpen) {
      if (!silent) {
        state.pending += "$";
      }
      state.pos += 1;
      return true;
    }
  }

  const start = state.pos + marker.markup.length;
  let match = start;

  if (marker.markup === "$") {
    while ((match = state.src.indexOf(marker.close, match)) !== -1) {
      let position = match - 1;
      while (state.src[position] === "\\") {
        position -= 1;
      }

      if ((match - position) % 2 === 1) {
        break;
      }

      match += 1;
    }
  } else {
    match = state.src.indexOf(marker.close, start);
  }

  if (match === -1) {
    if (!silent) {
      state.pending += marker.markup;
    }
    state.pos = start;
    return true;
  }

  if (marker.markup === "$" && match - start === 0) {
    if (!silent) {
      state.pending += "$$";
    }
    state.pos = start + 1;
    return true;
  }

  if (marker.markup === "$") {
    const endDelimiter = isValidDelimiter(state, match);
    if (!endDelimiter.canClose) {
      if (!silent) {
        state.pending += "$";
      }
      state.pos = start;
      return true;
    }
  }

  if (!silent) {
    const token = state.push("math_inline", "math", 0);
    token.markup = marker.markup;
    token.content = state.src.slice(start, match);
  }

  state.pos = match + marker.close.length;
  return true;
}

function mathBlock(state: any, startLine: number, endLine: number, silent: boolean): boolean {
  let position = state.bMarks[startLine] + state.tShift[startLine];
  let max = state.eMarks[startLine];

  const marker = getBlockMathMarker(state, position);
  if (!marker) {
    return false;
  }

  position += marker.markup.length;
  let firstLine = state.src.slice(position, max);
  let lastLine = "";
  let nextLine = startLine;
  let found = false;

  if (silent) {
    return true;
  }

  if (firstLine.trim().endsWith(marker.close)) {
    firstLine = firstLine.trim().slice(0, -marker.close.length);
    found = true;
  }

  while (!found) {
    nextLine += 1;
    if (nextLine >= endLine) {
      break;
    }

    position = state.bMarks[nextLine] + state.tShift[nextLine];
    max = state.eMarks[nextLine];

    if (position < max && state.tShift[nextLine] < state.blkIndent) {
      break;
    }

    if (state.src.slice(position, max).trim().endsWith(marker.close)) {
      const lastPosition = state.src.slice(0, max).lastIndexOf(marker.close);
      lastLine = state.src.slice(position, lastPosition);
      found = true;
    }
  }

  state.line = nextLine + 1;

  const token = state.push("math_block", "math", 0);
  token.block = true;
  token.content =
    (firstLine.trim() ? `${firstLine}\n` : "") +
    state.getLines(startLine + 1, nextLine, state.tShift[startLine], true) +
    (lastLine.trim() ? lastLine : "");
  token.map = [startLine, state.line];
  token.markup = marker.markup;
  return true;
}

export default function markdownItKatex(md: MarkdownIt, options: MarkdownItKatexOptions = {}) {
  md.inline.ruler.before("escape", "math_inline", mathInline);
  md.block.ruler.after("blockquote", "math_block", mathBlock, {
    alt: ["paragraph", "reference", "blockquote", "list"]
  });

  md.renderer.rules.math_inline = (tokens, idx) =>
    renderKatex(tokens[idx].content, options, false);
  md.renderer.rules.math_block = (tokens, idx) =>
    `<p>${renderKatex(tokens[idx].content, options, true)}</p>\n`;
}
