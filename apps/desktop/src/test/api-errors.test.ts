import { describe, expect, it } from "vitest";
import { applicationErrorMessage } from "../api/errors";

describe("applicationErrorMessage", () => {
  it("preserves structured backend error messages", () => {
    expect(applicationErrorMessage({ code: "provider_rate_limited", message: "模型服务限流" })).toBe("模型服务限流");
  });

  it("keeps compatibility with legacy string errors", () => {
    expect(applicationErrorMessage("读取项目失败")).toBe("读取项目失败");
  });
});
