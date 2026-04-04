import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import type { FileAttachment, ImageAttachment } from "./transport/message-types";
import { isTauriWindowAvailable } from "./tauri-window";

export type ComposerImageAttachment = ImageAttachment & {
  name: string;
};

export type ComposerFileAttachment = FileAttachment;

export type ComposerAttachmentSelection = {
  images: ComposerImageAttachment[];
  files: ComposerFileAttachment[];
};

const MIME_TYPE_BY_EXTENSION: Record<string, string> = {
  csv: "text/csv",
  gif: "image/gif",
  jpeg: "image/jpeg",
  jpg: "image/jpeg",
  json: "application/json",
  md: "text/markdown",
  pdf: "application/pdf",
  png: "image/png",
  svg: "image/svg+xml",
  txt: "text/plain",
  webp: "image/webp"
};

export function mergeComposerImages(
  existing: ComposerImageAttachment[],
  incoming: ComposerImageAttachment[]
) {
  if (incoming.length === 0) {
    return existing;
  }

  return [
    ...existing,
    ...incoming.filter(
      (nextImage) =>
        !existing.some(
          (currentImage) =>
            currentImage.base64 === nextImage.base64 &&
            currentImage.mimeType === nextImage.mimeType &&
            currentImage.name === nextImage.name
        )
    )
  ];
}

export function mergeComposerFiles(
  existing: ComposerFileAttachment[],
  incoming: ComposerFileAttachment[]
) {
  if (incoming.length === 0) {
    return existing;
  }

  return [
    ...existing,
    ...incoming.filter(
      (nextFile) =>
        !existing.some(
          (currentFile) =>
            currentFile.base64 === nextFile.base64 &&
            currentFile.mimeType === nextFile.mimeType &&
            currentFile.name === nextFile.name
        )
    )
  ];
}

export async function readComposerAttachmentsFromFiles(
  files: File[]
): Promise<ComposerAttachmentSelection> {
  const attachments = await Promise.all(files.map(readBrowserFileAttachment));
  return splitComposerAttachments(attachments);
}

export async function pickComposerAttachments(): Promise<ComposerAttachmentSelection> {
  if (!isTauriWindowAvailable()) {
    return { images: [], files: [] };
  }

  const selectedPaths = await openDialog({
    multiple: true,
    directory: false,
    pickerMode: "document",
    fileAccessMode: "copy",
    title: "选择附件"
  });
  const paths = normalizeSelectedPaths(selectedPaths);
  if (paths.length === 0) {
    return { images: [], files: [] };
  }

  const attachments = await Promise.all(paths.map(readTauriFileAttachment));
  return splitComposerAttachments(attachments);
}

type ComposerAttachment = ComposerImageAttachment | ComposerFileAttachment;

async function readBrowserFileAttachment(file: File): Promise<ComposerAttachment> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  const name = file.name || "attachment";
  const mimeType = normalizeMimeType(file.type, name);
  return buildComposerAttachment(name, mimeType, bytes);
}

async function readTauriFileAttachment(path: string): Promise<ComposerAttachment> {
  const bytes = await readFile(path);
  const name = getFilenameFromPath(path);
  const mimeType = normalizeMimeType(null, name);
  return buildComposerAttachment(name, mimeType, bytes);
}

function buildComposerAttachment(
  name: string,
  mimeType: string,
  bytes: Uint8Array
): ComposerAttachment {
  const base64 = bytesToBase64(bytes);
  if (mimeType.startsWith("image/")) {
    return { name, base64, mimeType };
  }

  return { name, base64, mimeType };
}

function splitComposerAttachments(attachments: ComposerAttachment[]): ComposerAttachmentSelection {
  const images: ComposerImageAttachment[] = [];
  const files: ComposerFileAttachment[] = [];

  for (const attachment of attachments) {
    if (attachment.mimeType.startsWith("image/")) {
      images.push(attachment as ComposerImageAttachment);
    } else {
      files.push(attachment as ComposerFileAttachment);
    }
  }

  return { images, files };
}

function normalizeSelectedPaths(selectedPaths: string | string[] | null): string[] {
  if (!selectedPaths) {
    return [];
  }

  return Array.isArray(selectedPaths) ? selectedPaths : [selectedPaths];
}

function normalizeMimeType(rawMimeType: string | null | undefined, name: string) {
  const mimeType = rawMimeType?.trim().toLowerCase();
  if (mimeType) {
    return mimeType;
  }

  const extension = name.split(".").pop()?.trim().toLowerCase();
  return (extension && MIME_TYPE_BY_EXTENSION[extension]) || "application/octet-stream";
}

function getFilenameFromPath(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || "attachment";
}

function bytesToBase64(bytes: Uint8Array) {
  if (typeof Buffer !== "undefined") {
    return Buffer.from(bytes).toString("base64");
  }

  let binary = "";
  const chunkSize = 0x8000;
  for (let index = 0; index < bytes.length; index += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize));
  }
  return btoa(binary);
}
