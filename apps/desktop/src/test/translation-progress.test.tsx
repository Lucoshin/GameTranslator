import { act, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { TranslationProgress } from "../features/translation/TranslationProgress";

it("renders backend progress instead of fake completion data", () => {
  render(
    <TranslationProgress
      result={null}
      demo={false}
      loading
      error={null}
      progress={{ phase: "translating", completed: 25, total: 100, failed: 1, warningFindings: 2, blockingFindings: 0, message: "已完成模型批次 25 / 100" }}
      logs={[{ time: "09:42:12", message: "已完成模型批次 25 / 100" }]}
      onReview={vi.fn()}
      onExport={vi.fn()}
    />
  );

  expect(screen.getByText("25%")).toBeVisible();
  expect(screen.getByText("25 / 100 · 已运行 00:00")).toBeVisible();
  expect(screen.getAllByText("已完成模型批次 25 / 100")).toHaveLength(2);
  expect(screen.queryByText("完成 Map023.json · 48 个片段")).toBeNull();
});

afterEach(() => vi.useRealTimers());

it("shows a live elapsed timer while translation is running", () => {
  vi.useFakeTimers();
  render(
    <TranslationProgress
      result={null}
      demo={false}
      loading
      error={null}
      progress={{ phase: "translating", completed: 0, total: 100, failed: 0, warningFindings: 0, blockingFindings: 0, message: "已启动 8 路并发请求" }}
      logs={[]}
      onReview={vi.fn()}
      onExport={vi.fn()}
    />
  );

  act(() => vi.advanceTimersByTime(3_000));

  expect(screen.getByText("0 / 100 · 已运行 00:03")).toBeVisible();
  expect(screen.getByText("已启动 8 路并发请求")).toBeVisible();
});
