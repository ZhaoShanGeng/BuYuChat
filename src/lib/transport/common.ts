/**
 * Tauri transport 层的公共类型、错误归一化与序列化辅助函数。
 *
 * 这个文件只放跨资源共用的最小能力：
 * 1. `AppError`：前端统一错误结构。
 * 2. `toAppError`：把未知异常收敛成可展示的错误对象。
 * 3. `toOptionalValue`：保留补丁语义中的 `null`，只把 `undefined` 视为“未提供字段”。
 */

/**
 * 前端统一使用的错误模型。
 */
export type AppError = {
  errorCode: string;
  message: string;
};

/**
 * Tauri IPC 返回的原始错误载荷。
 */
type RawError = {
  error_code?: string;
  message?: string;
};

/**
 * 将未知错误归一化为前端统一错误结构。
 */
export function toAppError(error: unknown): AppError {
  const fallback: AppError = {
    errorCode: "INTERNAL_ERROR",
    message: "unexpected client error"
  };

  if (!error || typeof error !== "object") {
    return fallback;
  }

  const raw = error as RawError;
  return {
    errorCode: raw.error_code ?? fallback.errorCode,
    message: raw.message ?? fallback.message
  };
}

/**
 * 将补丁字段转换为可传给后端的值。
 *
 * 这里必须保留 `null`，因为后端大量使用 `Option<Option<T>>`
 * 来区分“未提供字段”和“显式清空字段”。
 */
export function toOptionalValue<T>(value: T | null | undefined): T | null | undefined {
  return value === undefined ? undefined : value;
}
