import { fireEvent, render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => undefined) }));

beforeEach(() => {
  localStorage.clear();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(undefined);
});

function openDemoProject() {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "载入演示项目" }));
}

describe("project flow", () => {
  it("configures the model from the startup screen", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "主界面配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
  });

  it("opens and scans a real project through Tauri", async () => {
    vi.mocked(invoke).mockResolvedValue({
      projectPath: "D:\\Games\\RealGame",
      projectName: "RealGame",
      engine: "RPG Maker MZ",
      segmentCount: 9,
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "选择游戏目录" }));

    expect(await screen.findByRole("heading", { name: "RealGame" })).toBeVisible();
    expect(screen.getAllByText("9")[0]).toBeVisible();
    expect(screen.queryByText("演示数据，不会读取或修改本地文件")).toBeNull();
    expect(invoke).toHaveBeenCalledWith("select_and_scan_project");
  });

  it("saves provider credentials and translates a real project", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        projectPath: "D:\\Games\\RealGame",
        projectName: "RealGame",
        engine: "RPG Maker MZ",
        segmentCount: 1,
      })
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce({
        items: [{
          id: "Map001.json:events[1].pages[0].list[1]",
          source: "やっと着いた。 \\V[1]",
          target: "终于到了。 \\V[1]",
          speaker: "アリス",
          sourceFile: "D:\\Games\\RealGame\\data\\Map001.json",
          qa: "passed",
        }],
        warningFindings: 0,
        blockingFindings: 0,
        failedSegmentIds: [],
      })
      .mockResolvedValueOnce({
        outputPath: "D:\\Exports\\RealGame-zhCN",
        fileCount: 1,
      });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择游戏目录" }));
    await screen.findByRole("heading", { name: "RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));
    fireEvent.change(screen.getByLabelText("API Key"), { target: { value: "secret" } });
    fireEvent.change(screen.getByLabelText("模型名称"), { target: { value: "test-model" } });
    fireEvent.change(screen.getByLabelText("性能模式"), { target: { value: "fast" } });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await screen.findByText("test-model");

    fireEvent.change(screen.getByLabelText("源语言"), { target: { value: "ja-JP" } });
    fireEvent.change(screen.getByLabelText("目标语言"), { target: { value: "en-US" } });

    fireEvent.click(screen.getByRole("button", { name: "开始汉化" }));

    expect(await screen.findByRole("heading", { name: "翻译任务" })).toBeVisible();
    expect(screen.getByText("1 / 1")).toBeVisible();
    expect(invoke).toHaveBeenNthCalledWith(2, "save_provider_configuration", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls[1][1]).toMatchObject({ provider: { performance: "fast" } });
    expect(invoke).toHaveBeenNthCalledWith(3, "translate_project", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls[2][1]).toMatchObject({
      input: {
        sourceLanguage: { code: "ja-JP", name: "日语" },
        targetLanguage: { code: "en-US", name: "英语" },
      },
    });

    fireEvent.click(screen.getByRole("button", { name: "校对文本" }));
    expect(screen.getByText("英语")).toBeVisible();
    fireEvent.change(screen.getByDisplayValue("终于到了。 \\V[1]"), {
      target: { value: "总算到了。 \\V[1]" },
    });
    fireEvent.click(screen.getByRole("button", { name: "导出补丁" }));
    fireEvent.click(screen.getByRole("button", { name: "生成汉化补丁" }));

    expect(await screen.findByText("D:\\Exports\\RealGame-zhCN")).toBeVisible();
    expect(invoke).toHaveBeenNthCalledWith(4, "export_translation_patch", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls[3][1]).toMatchObject({
      input: {
        targetLanguage: { code: "en-US", name: "英语" },
        items: [{ target: "总算到了。 \\V[1]" }],
      },
    });
  });

  it("opens a clearly labelled demo project", () => {
    openDemoProject();

    expect(screen.getByRole("heading", { name: "月影神殿" })).toBeVisible();
    expect(screen.getByText("演示数据，不会读取或修改本地文件")).toBeVisible();
    expect(screen.getAllByText("RPG Maker MZ")[0]).toBeVisible();
  });

  it("configures a model before starting translation", async () => {
    openDemoProject();
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
    fireEvent.change(screen.getByLabelText("模型名称"), {
      target: { value: "deepseek-chat" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));

    expect(await screen.findByText("deepseek-chat")).toBeVisible();
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
