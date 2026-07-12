import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { ProviderDrawer } from "../features/providers/ProviderDrawer";

it("uses a preset Base URL without preventing custom edits", () => {
  render(
    <ProviderDrawer
      open
      current={null}
      onClose={vi.fn()}
      onSave={vi.fn()}
    />,
  );

  fireEvent.change(screen.getByLabelText("常见格式"), {
    target: { value: "https://api.deepseek.com/v1" },
  });
  expect(screen.getByLabelText("Base URL")).toHaveValue("https://api.deepseek.com/v1");

  fireEvent.change(screen.getByLabelText("Base URL"), {
    target: { value: "https://gateway.example.com/openai/v1" },
  });
  expect(screen.getByLabelText("Base URL")).toHaveValue("https://gateway.example.com/openai/v1");
});
