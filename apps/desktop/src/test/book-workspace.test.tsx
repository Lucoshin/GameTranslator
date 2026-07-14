import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { BookProject } from "../features/books/contracts";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => undefined) }));

const defaultPreferences = {
  recentProjectPath: null,
  sourceLanguage: { code: "auto", name: "自动检测" },
  targetLanguage: { code: "zh-CN", name: "简体中文" },
};

const book: BookProject = {
  id: "book-1",
  sourcePath: "D:\\Books\\海边来信.epub",
  title: "海边来信",
  format: "epub",
  sourceLanguage: "auto",
  targetLanguage: "zh-CN",
  publication: {
    author: "原作者",
    translator: "测试译者",
    publisher: "",
    isbn: "",
    copyright: "",
    coverPath: "",
    printPreset: "large32",
  },
  chapters: [{
    id: "chapter-1",
    title: "潮声背后",
    segments: [
      { id: "segment-1", source: "The harbor was quiet.", translation: "港口很安静。", status: "draft", qaNote: null, terms: [] },
      { id: "segment-2", source: "A bell rang.", translation: "钟声响起。", status: "issue", qaNote: "UnchangedTranslation", terms: [] },
      { id: "segment-3", source: "She opened the letter.", translation: "她拆开了信。", status: "draft", qaNote: null, terms: [] },
    ],
  }],
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation((command) => {
    if (command === "load_provider_configuration") return Promise.resolve(null);
    if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
    if (command === "list_resumable_tasks") return Promise.resolve([]);
    if (command === "list_book_projects") return Promise.resolve([book]);
    if (command === "save_book_project") return Promise.resolve(undefined);
    if (command === "list_book_export_history") return Promise.resolve([]);
    return Promise.resolve(undefined);
  });
  window.location.hash = "";
});

describe("书籍工作台桌面集成", () => {
  it("启动后直接进入统一项目中心", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "所有项目" })).toBeVisible();
    expect(screen.getByRole("navigation", { name: "应用导航" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "进入书籍翻译工作台" })).toBeNull();
  });

  it("打开书籍后保留统一阶段导航并移除书籍专属导航", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));

    const navigation = screen.getByRole("navigation", { name: "应用导航" });
    for (const label of ["项目", "概览", "翻译", "校对", "导出"]) {
      expect(navigation).toHaveTextContent(label);
    }
    expect(screen.queryByRole("navigation", { name: "书籍项目导航" })).toBeNull();
  });

  it("阅读编辑默认显示每个段落的原文而无需逐段选中", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));

    const editor = within(screen.getByRole("region", { name: "阅读编辑" }));
    expect(editor.getByText("The harbor was quiet.")).toBeVisible();
    expect(editor.getByText("A bell rang.")).toBeVisible();
    expect(editor.getByText("She opened the letter.")).toBeVisible();
  });

  it("从统一项目中心打开持久化书籍并返回", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "所有项目" })).toBeVisible();
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));

    const nav = screen.getByRole("navigation", { name: "应用导航" });
    for (const label of ["项目", "概览", "翻译", "校对", "导出"]) expect(nav).toHaveTextContent(label);

    fireEvent.click(screen.getByRole("button", { name: "返回项目中心" }));
    expect(screen.getByRole("heading", { name: "所有项目" })).toBeVisible();
    expect(screen.getByRole("button", { name: "选择内容目录" })).toBeVisible();
  });

  it("切换校对模式后保留当前段落并支持确认到下一段", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));
    fireEvent.click(screen.getByRole("button", { name: "选择段落 2" }));
    fireEvent.click(screen.getByRole("button", { name: "逐段校对" }));

    expect(screen.getByRole("button", { name: "选择段落 2" })).toHaveAttribute("aria-current", "true");
    fireEvent.keyDown(screen.getByRole("textbox", { name: "段落 2 译文" }), { key: "Enter", ctrlKey: true });
    expect(screen.getByRole("button", { name: "选择段落 3" })).toHaveAttribute("aria-current", "true");
  });

  it("空项目中心通过真实导入命令进入书籍工作台", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "list_resumable_tasks") return Promise.resolve([]);
      if (command === "list_book_projects") return Promise.resolve([]);
      if (command === "import_book_project") return Promise.resolve(book);
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "选择书籍文件" }));

    expect(await screen.findByRole("heading", { name: "海边来信" })).toBeVisible();
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("import_book_project"));
  });

  it("在书籍工作台中打开共享模型配置", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));

    expect(screen.getByRole("dialog", { name: "模型接入" })).toBeVisible();
  });

  it("通过统一阶段导航浏览书籍概览、校对和导出", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "list_resumable_tasks") return Promise.resolve([]);
      if (command === "list_book_projects") return Promise.resolve([book]);
      if (command === "list_book_export_history") return Promise.resolve([]);
      if (command === "export_book_project") return Promise.resolve({ id: "export-1", projectId: "book-1", bookTitle: "海边来信", format: "markdown", outputPath: "D:\\Books\\海边来信-zh-CN.md", targetLanguage: "zh-CN", exportedAtUnixMs: 1, profile: { printPreset: "large32", includePageNumbers: true, chapterStartsNewPage: true } });
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));

    fireEvent.click(screen.getByRole("button", { name: "概览" }));
    expect(screen.getByRole("heading", { name: "书籍概览" })).toBeVisible();
    expect(screen.getByText("EPUB")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "校对" }));
    expect(screen.getByRole("region", { name: "逐段校对" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "导出" }));
    fireEvent.click(screen.getByRole("button", { name: "导出 Markdown" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("export_book_project", { request: expect.objectContaining({ project: expect.objectContaining({ id: "book-1" }), format: "markdown" }) }));
  });

  it("出版导出页始终提供四种明确可见的导出操作", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));
    fireEvent.click(screen.getByRole("button", { name: "导出" }));

    for (const label of ["导出 DOCX", "导出 EPUB", "导出 PDF", "导出 Markdown"]) {
      expect(screen.getByRole("button", { name: label })).toBeVisible();
    }
    expect(screen.getByRole("combobox", { name: "印刷成品尺寸" })).toHaveValue("large32");
  });

  it("书籍历史显示真实导出记录并可定位文件", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve(null);
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "list_resumable_tasks") return Promise.resolve([]);
      if (command === "list_book_projects") return Promise.resolve([book]);
      if (command === "list_book_export_history") return Promise.resolve([{ id: "export-1", projectId: "book-1", bookTitle: "海边来信", format: "docx", outputPath: "D:\\Books\\海边来信.docx", targetLanguage: "zh-CN", exportedAtUnixMs: 1_720_000_000_000, profile: { printPreset: "large32", includePageNumbers: true, chapterStartsNewPage: true } }]);
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));
    fireEvent.click(screen.getByRole("button", { name: "历史" }));

    expect(await screen.findByText("DOCX")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "在文件夹中显示" }));
    expect(invoke).toHaveBeenCalledWith("open_book_export_location", { path: "D:\\Books\\海边来信.docx" });
  });

  it("在书籍编辑阶段选择语言并将书籍语言传给翻译命令", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "load_provider_configuration") return Promise.resolve({ kind: "openai", baseUrl: "https://api.example.com/v1", model: "book-model", performance: "fast" });
      if (command === "load_desktop_preferences") return Promise.resolve(defaultPreferences);
      if (command === "list_resumable_tasks") return Promise.resolve([]);
      if (command === "list_book_projects") return Promise.resolve([book]);
      if (command === "save_book_project") return Promise.resolve(undefined);
      if (command === "translate_book_project") return Promise.resolve({ ...book, sourceLanguage: "en-US", targetLanguage: "ja-JP" });
      return Promise.resolve(undefined);
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "打开《海边来信》" }));

    fireEvent.change(screen.getByRole("combobox", { name: "书籍源语言" }), { target: { value: "en-US" } });
    fireEvent.change(screen.getByRole("combobox", { name: "书籍目标语言" }), { target: { value: "ja-JP" } });
    fireEvent.click(screen.getByRole("button", { name: "翻译本章" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("translate_book_project", {
      input: expect.objectContaining({
        project: expect.objectContaining({ sourceLanguage: "en-US", targetLanguage: "ja-JP" }),
        sourceLanguage: { code: "en-US", name: "英语" },
        targetLanguage: { code: "ja-JP", name: "日语" },
      }),
    }));
  });
});
