import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => undefined) }));

const defaultPreferences = { recentProjectPath: null, sourceLanguage: { code: "auto", name: "自动检测" }, targetLanguage: { code: "zh-CN", name: "简体中文" } };

beforeEach(() => {
  localStorage.clear();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation((command) => command === "load_provider_configuration" ? Promise.resolve(null) : command === "load_desktop_preferences" ? Promise.resolve(defaultPreferences) : Promise.resolve(undefined));
  vi.mocked(listen).mockResolvedValue(() => undefined);
});

function openProject() {
  const project = {
    projectPath: "D:\\Games\\RealGame",
    projectName: "RealGame",
    engine: "RPG Maker MZ",
    segmentCount: 2,
  };
  vi.mocked(invoke).mockImplementation((command) => command === "load_provider_configuration" ? Promise.resolve(null) : command === "load_desktop_preferences" ? Promise.resolve(defaultPreferences) : command === "select_and_scan_project" ? Promise.resolve(project) : Promise.resolve(undefined));
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
}

describe("project flow", () => {
  it("restores provider, recent project, and languages from backend persistence", async () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem");
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve({ kind: "openai", baseUrl: "https://api.deepseek.com/v1", model: "deepseek-chat", performance: "fast" });
      if (command === "load_desktop_preferences") return Promise.resolve({ recentProjectPath: "D:\\Games\\Moon", sourceLanguage: { code: "ja-JP", name: "日语" }, targetLanguage: { code: "zh-CN", name: "简体中文" } });
      if (command === "scan_project_path") return Promise.resolve({ projectPath: "D:\\Games\\Moon", projectName: "Moon", engine: "Ren'Py", segmentCount: 3 });
      return Promise.resolve(undefined);
    });

    render(<App />);

    expect(await screen.findByRole("heading", { name: "Moon" })).toBeVisible();
    expect(screen.getByRole("combobox", { name: "源语言" })).toHaveValue("ja-JP");
    expect(screen.getByText("deepseek-chat")).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("scan_project_path", { projectPath: "D:\\Games\\Moon" });
    expect(getItem).not.toHaveBeenCalled();
    getItem.mockRestore();
  });

  it("shows persistence load failures after restoring a project", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.reject(new Error("configuration is invalid"));
      if (command === "load_desktop_preferences") return Promise.resolve({ ...defaultPreferences, recentProjectPath: "D:\\Games\\Moon" });
      if (command === "scan_project_path") return Promise.resolve({ projectPath: "D:\\Games\\Moon", projectName: "Moon", engine: "Ren'Py", segmentCount: 3 });
      return Promise.resolve(undefined);
    });

    render(<App />);

    expect(await screen.findByRole("heading", { name: "Moon" })).toBeVisible();
    expect(screen.getByRole("alert")).toHaveTextContent("读取模型配置失败");
  });

  it("configures the model from the startup screen", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "主界面配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
  });

  it("opens and scans a real project through Tauri", async () => {
    vi.mocked(invoke).mockImplementation((command) => command === "load_provider_configuration" ? Promise.resolve(null) : command === "load_desktop_preferences" ? Promise.resolve(defaultPreferences) : Promise.resolve({
      projectPath: "D:\\Games\\RealGame",
      projectName: "RealGame",
      engine: "RPG Maker MZ",
      segmentCount: 9,
    }));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));

    expect(await screen.findByRole("heading", { name: "RealGame" })).toBeVisible();
    expect(screen.getAllByText("9")[0]).toBeVisible();
    expect(screen.queryByText("演示数据，不会读取或修改本地文件")).toBeNull();
    expect(invoke).toHaveBeenCalledWith("select_and_scan_project");
  });

  it("saves provider credentials and translates a real project", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "select_and_scan_project") return Promise.resolve({
        projectPath: "D:\\Games\\RealGame",
        projectName: "RealGame",
        engine: "RPG Maker MZ",
        segmentCount: 1,
      });
      if (command === "save_provider_configuration") return Promise.resolve(undefined);
      if (command === "translate_project") return Promise.resolve({
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
      });
      if (command === "export_translation_patch") return Promise.resolve({
        outputPath: "D:\\Exports\\RealGame-zhCN",
        fileCount: 1,
      });
      if (command === "install_translation_patch") return Promise.resolve({
        installedPath: "D:\\Games\\RealGame\\game\\tl\\en_US",
        fileCount: 2,
      });
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));
    fireEvent.change(screen.getByLabelText("API Key"), { target: { value: "secret" } });
    fireEvent.change(screen.getByLabelText("模型名称"), { target: { value: "test-model" } });
    fireEvent.change(screen.getByLabelText("性能模式"), { target: { value: "fast" } });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));
    await screen.findByText("test-model");

    fireEvent.change(screen.getByLabelText("源语言"), { target: { value: "ja-JP" } });
    fireEvent.change(screen.getByLabelText("目标语言"), { target: { value: "en-US" } });

    fireEvent.click(screen.getByRole("button", { name: "开始翻译" }));

    expect(await screen.findByRole("heading", { name: "翻译任务" })).toBeVisible();
    expect(screen.getByText("1 / 1")).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("save_provider_configuration", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls.find(([command]) => command === "save_provider_configuration")?.[1]).toMatchObject({ provider: { performance: "fast" } });
    expect(invoke).toHaveBeenCalledWith("translate_project", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls.find(([command]) => command === "translate_project")?.[1]).toMatchObject({
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
    expect(screen.getByRole("button", { name: "应用校对修改（1）" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "导出补丁" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "应用校对修改（1）" }));
    expect(screen.getByRole("button", { name: "应用校对修改（0）" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "导出补丁" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "导出补丁" }));
    fireEvent.click(screen.getByRole("button", { name: "生成翻译补丁" }));

    expect(await screen.findByText("D:\\Exports\\RealGame-zhCN")).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("export_translation_patch", expect.any(Object));
    expect(vi.mocked(invoke).mock.calls.find(([command]) => command === "export_translation_patch")?.[1]).toMatchObject({
      input: {
        targetLanguage: { code: "en-US", name: "英语" },
        items: [{ target: "总算到了。 \\V[1]" }],
      },
    });
    fireEvent.click(screen.getByRole("button", { name: "安装到当前内容" }));
    expect(await screen.findByText("翻译已安装，重新启动游戏后生效")).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("install_translation_patch", expect.any(Object));
  });

  it("starts the backend even when event listener registration stalls", async () => {
    vi.mocked(listen).mockReturnValue(new Promise(() => undefined));
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve({ kind: "openai", baseUrl: "https://api.deepseek.com", model: "deepseek-v4-flash", performance: "fast" });
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "select_and_scan_project") {
        return Promise.resolve({
          projectPath: "D:\\Games\\RealGame",
          projectName: "RealGame",
          engine: "Ren'Py",
          segmentCount: 10,
        });
      }
      return new Promise(() => undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });

    fireEvent.click(screen.getByRole("button", { name: "开始翻译" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("translate_project", expect.any(Object)));
  });

  it("does not expose a demo project or sample game data", () => {
    render(<App />);

    expect(screen.queryByRole("button", { name: "载入演示项目" })).toBeNull();
    expect(screen.queryByText("月影神殿")).toBeNull();
    expect(screen.queryByText("演示数据，不会读取或修改本地文件")).toBeNull();
  });

  it("selects another game directly from project overview", async () => {
    openProject();
    await screen.findByRole("heading", { name: "RealGame" });
    vi.mocked(invoke).mockImplementation((command) => command === "load_provider_configuration" ? Promise.resolve(null) : command === "load_desktop_preferences" ? Promise.resolve(defaultPreferences) : command === "select_and_scan_project" ? Promise.resolve({
      projectPath: "D:\\Games\\AnotherGame",
      projectName: "AnotherGame",
      engine: "Ren'Py",
      segmentCount: 12,
    }) : Promise.resolve(undefined));

    fireEvent.click(screen.getByRole("button", { name: "概览页选择内容目录" }));

    expect(await screen.findByRole("heading", { name: "AnotherGame" })).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("select_and_scan_project");
  });

  it("configures a model before starting translation", async () => {
    openProject();
    await screen.findByRole("heading", { name: "RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
    fireEvent.change(screen.getByLabelText("模型名称"), {
      target: { value: "deepseek-chat" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存配置" }));

    expect(await screen.findByText("deepseek-chat")).toBeVisible();
  });

  it("filters review rows from a real translation result by speaker or location", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve({ kind: "openai", baseUrl: "https://api.example.com/v1", model: "test-model", performance: "balanced" });
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "select_and_scan_project") return Promise.resolve({
        projectPath: "D:\\Games\\RealGame",
        projectName: "RealGame",
        engine: "RPG Maker MZ",
        segmentCount: 2,
      });
      if (command === "translate_project") return Promise.resolve({
        items: [
          { id: "1", source: "やっと着いた。", target: "终于到了。", speaker: "爱丽丝", sourceFile: "Map001.json", qa: "passed" },
          { id: "2", source: "通行証を見せろ。", target: "出示通行证。", speaker: "守衛", sourceFile: "Map002.json", qa: "passed" },
        ],
        warningFindings: 0,
        blockingFindings: 0,
        failedSegmentIds: [],
      });
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "开始翻译" }));
    await screen.findByRole("heading", { name: "翻译任务" });
    fireEvent.click(screen.getByRole("button", { name: "校对文本" }));

    fireEvent.change(screen.getByRole("combobox", { name: "按角色或位置筛选" }), {
      target: { value: "speaker:守衛" },
    });

    expect(screen.getByText("通行証を見せろ。")).toBeVisible();
    expect(screen.queryByText("やっと着いた。")).toBeNull();
  });

  it("toggles between the global home and project overview without closing the project", async () => {
    openProject();
    await screen.findByRole("heading", { name: "RealGame" });

    fireEvent.click(screen.getByRole("button", { name: "返回全局首页" }));
    expect(screen.getByRole("button", { name: "选择内容目录" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "返回项目概览" }));
    expect(screen.getByRole("heading", { name: "RealGame" })).toBeVisible();
  });

  it("opens the empty project overview from the startup brand mark without selecting a directory", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "进入项目概览" }));

    expect(await screen.findByRole("heading", { name: "开始一个项目" })).toBeVisible();
    expect(invoke).not.toHaveBeenCalledWith("select_and_scan_project");
  });

  it("opens all persisted patch history without selecting a project", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "list_patch_history") return Promise.resolve([{
        id: "patch-global",
        projectPath: "D:\\Games\\Moon",
        patchPath: "D:\\Patches\\Moon-zh-CN",
        targetLanguage: "zh-CN",
        fileCount: 2,
        exportedAtUnixMs: 1_700_000_000_000,
        installedAtUnixMs: null,
      }]);
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "进入项目概览" }));
    await screen.findByRole("heading", { name: "开始一个项目" });

    expect(screen.getByRole("button", { name: "历史" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "历史" }));

    expect(await screen.findByRole("heading", { name: "补丁历史" })).toBeVisible();
    expect(await screen.findByText("Moon-zh-CN")).toBeVisible();
    expect(screen.getByText(/Moon · zh-CN/)).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("list_patch_history", { projectPath: null });
  });

  it("shows extracted source text and a start action before translation", async () => {
    vi.mocked(invoke).mockImplementation((command) => command === "load_provider_configuration" ? Promise.resolve(null) : command === "load_desktop_preferences" ? Promise.resolve(defaultPreferences) : Promise.resolve({
      projectPath: "D:\\Games\\RealGame",
      projectName: "RealGame",
      engine: "RPG Maker MZ",
      segmentCount: 1,
      previewItems: [{
        id: "Map001.json::events[1]",
        source: "やっと着いた。",
        target: "",
        speaker: "アリス",
        sourceFile: "Map001.json",
        qa: "passed",
      }],
    }));
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });

    fireEvent.click(screen.getByRole("button", { name: "任务" }));
    expect(screen.getByText("准备开始翻译")).toBeVisible();
    expect(screen.getByRole("button", { name: /开始翻译/ })).toBeEnabled();
    expect(screen.queryByText("正在请求模型")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "校对" }));
    expect(screen.getByText("やっと着いた。")).toBeVisible();
    expect(screen.getByLabelText("翻译 やっと着いた。")).toBeDisabled();
  });

  it("loads and uninstalls an installed patch from project history", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "select_and_scan_project") return Promise.resolve({
        projectPath: "D:\\Games\\RealGame",
        projectName: "RealGame",
        engine: "Ren'Py",
        segmentCount: 2,
      });
      if (command === "list_patch_history") return Promise.resolve([{
        id: "patch-1",
        projectPath: "D:\\Games\\RealGame",
        patchPath: "D:\\Patches\\RealGame-zh-CN",
        targetLanguage: "zh-CN",
        fileCount: 2,
        exportedAtUnixMs: 1_700_000_000_000,
        installedAtUnixMs: 1_700_000_100_000,
      }]);
      if (command === "uninstall_translation_patch") return Promise.resolve({ restoredFileCount: 1, removedFileCount: 1 });
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });

    fireEvent.click(screen.getByRole("button", { name: "历史" }));

    expect(await screen.findByRole("heading", { name: "补丁历史" })).toBeVisible();
    expect(screen.getByText("RealGame-zh-CN")).toBeVisible();
    expect(invoke).toHaveBeenCalledWith("list_patch_history", { projectPath: "D:\\Games\\RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "卸载翻译补丁" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("uninstall_translation_patch", {
      input: { projectPath: "D:\\Games\\RealGame", id: "patch-1" },
    }));
    expect(screen.getByText("已卸载")).toBeVisible();
  });

  it("installs exported patches and deletes only uninstalled history records", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "select_and_scan_project") return Promise.resolve({ projectPath: "D:\\Games\\RealGame", projectName: "RealGame", engine: "Ren'Py", segmentCount: 2 });
      if (command === "list_patch_history") return Promise.resolve([
        { id: "patch-install", projectPath: "D:\\Games\\RealGame", patchPath: "D:\\Patches\\Install", targetLanguage: "zh-CN", fileCount: 2, exportedAtUnixMs: 1_700_000_000_000, installedAtUnixMs: null },
        { id: "patch-delete", projectPath: "D:\\Games\\RealGame", patchPath: "D:\\Patches\\Delete", targetLanguage: "zh-CN", fileCount: 1, exportedAtUnixMs: 1_700_000_100_000, installedAtUnixMs: null },
      ]);
      if (command === "install_translation_patch") return Promise.resolve({ installedPath: "D:\\Games\\RealGame\\game\\tl\\zh_CN", fileCount: 2 });
      if (command === "delete_patch_history_entry") return Promise.resolve(undefined);
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "选择内容目录" }));
    await screen.findByRole("heading", { name: "RealGame" });
    fireEvent.click(screen.getByRole("button", { name: "历史" }));

    await screen.findByRole("heading", { name: "补丁历史" });
    fireEvent.click(screen.getAllByRole("button", { name: "安装到当前内容" })[0]);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("install_translation_patch", expect.objectContaining({ input: expect.objectContaining({ patchPath: "D:\\Patches\\Install" }) })));
    fireEvent.click(screen.getAllByRole("button", { name: "删除历史记录" }).find((button) => !button.hasAttribute("disabled"))!);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("delete_patch_history_entry", { input: { projectPath: "D:\\Games\\RealGame", id: "patch-delete" } }));
  });
});
