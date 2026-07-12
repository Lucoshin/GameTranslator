import { act, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { TranslationProgress } from "../features/translation/TranslationProgress";

it("renders backend progress instead of fake completion data", () => {
  render(
    <TranslationProgress
      result={null}
      loading
      error={null}
      progress={{ phase: "translating", completed: 25, total: 100, failed: 1, warningFindings: 2, blockingFindings: 0, message: "已完成模型批次 25 / 100", concurrency: 16, throughput: 12.5, etaSeconds: 60 }}
      logs={[{ time: "09:42:12", message: "已完成模型批次 25 / 100" }]}
      sourceCount={100}
      onStart={vi.fn()}
      onReview={vi.fn()}
      onExport={vi.fn()}
    />
  );

  expect(screen.getByText("25%")).toBeVisible();
  expect(screen.getByText("25 / 100 · 已运行 00:00")).toBeVisible();
  expect(screen.getAllByText("已完成模型批次 25 / 100")).toHaveLength(2);
  expect(screen.queryByText("完成 Map023.json · 48 个片段")).toBeNull();
  expect(screen.getByText("16 路")).toBeVisible();
  expect(screen.getByText("12.5 片段/秒")).toBeVisible();
  expect(screen.getByText("预计 01:00")).toBeVisible();
});

afterEach(() => vi.useRealTimers());

it("shows a live elapsed timer while translation is running", () => {
  vi.useFakeTimers();
  render(
    <TranslationProgress
      result={null}
      loading
      error={null}
      progress={{ phase: "translating", completed: 0, total: 100, failed: 0, warningFindings: 0, blockingFindings: 0, message: "已启动 8 路并发请求" }}
      logs={[]}
      sourceCount={100}
      onStart={vi.fn()}
      onReview={vi.fn()}
      onExport={vi.fn()}
    />
  );

  act(() => vi.advanceTimersByTime(3_000));

  expect(screen.getByText("0 / 100 · 已运行 00:03")).toBeVisible();
  expect(screen.getByText("已启动 8 路并发请求")).toBeVisible();
});
