/**
 * 渠道页面状态辅助函数测试。
 */

import { describe, expect, it, vi } from "vitest";

import {
  createEmptyForm,
  humanizeError,
  removeChannel,
  submitChannelForm,
  verifyChannelConnectivity
} from "./channel-settings-state";

describe("channel settings state", () => {
  it("submits create flow and normalizes empty optional fields", async () => {
    const createChannel = vi.fn().mockResolvedValue(undefined);
    const notice = await submitChannelForm(
      {
        createChannel,
        updateChannel: vi.fn()
      },
      null,
      createEmptyForm()
    );

    expect(createChannel).toHaveBeenCalledTimes(1);
    expect(createChannel.mock.calls[0][0].apiKey).toBeNull();
    expect(notice).toEqual({ kind: "success", text: "渠道已创建" });
  });

  it("maps delete failures to translated error messages", async () => {
    const notice = await removeChannel(
      {
        deleteChannel: vi.fn().mockRejectedValue({
          error_code: "NOT_FOUND",
          message: "missing"
        })
      },
      "channel-1"
    );

    expect(notice).toEqual({ kind: "error", text: "资源不存在" });
  });

  it("returns connectivity success message", async () => {
    const notice = await verifyChannelConnectivity(
      {
        testChannel: vi.fn().mockResolvedValue({ success: true, message: "ok" })
      },
      "channel-1"
    );

    expect(notice).toEqual({ kind: "success", text: "渠道连通性验证成功" });
  });

  it("uses default error fallback for unknown codes", () => {
    expect(
      humanizeError({
        errorCode: "SOMETHING_ELSE",
        message: "unexpected"
      })
    ).toBe("系统内部错误，请重试");
  });
});
