import { render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
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
  expect(screen.getByText("25 / 100")).toBeVisible();
  expect(screen.getByText("已完成模型批次 25 / 100")).toBeVisible();
  expect(screen.queryByText("完成 Map023.json · 48 个片段")).toBeNull();
});
