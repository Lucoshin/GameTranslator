import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "../App";

function openDemoProject() {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "载入演示项目" }));
}

describe("project flow", () => {
  it("opens a clearly labelled demo project", () => {
    openDemoProject();

    expect(screen.getByRole("heading", { name: "月影神殿" })).toBeVisible();
    expect(screen.getByText("演示数据，不会读取或修改本地文件")).toBeVisible();
    expect(screen.getByText("RPG Maker MZ")).toBeVisible();
  });

  it("configures a model before starting translation", () => {
    openDemoProject();
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
    fireEvent.change(screen.getByLabelText("模型名称"), {
      target: { value: "deepseek-chat" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));

    expect(screen.getByText("deepseek-chat")).toBeVisible();
  });

  it("moves between translation review and export views", () => {
    openDemoProject();
    fireEvent.click(screen.getByRole("button", { name: "开始汉化" }));
    expect(screen.getByRole("heading", { name: "翻译任务" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "校对文本" }));
    expect(screen.getByRole("heading", { name: "文本校对" })).toBeVisible();
    expect(screen.getByText("终于到了。 \\V[1]")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "导出补丁" }));
    expect(screen.getByRole("heading", { name: "导出汉化补丁" })).toBeVisible();
    expect(screen.getByText("原游戏文件不会被直接修改")).toBeVisible();
  });
});

