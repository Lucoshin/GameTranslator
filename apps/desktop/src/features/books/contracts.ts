export type BookFormat = "txt" | "markdown" | "epub" | "docx";
export type SegmentStatus = "untranslated" | "draft" | "reviewed" | "issue";
export type BookExportFormat = "markdown" | "docx" | "epub" | "pdf";
export type PrintPreset = "large32" | "a5" | "sixteen";

export type PublicationMetadata = {
  author: string;
  translator: string;
  publisher: string;
  isbn: string;
  copyright: string;
  coverPath: string;
  printPreset: PrintPreset;
};

export type BookExportProfile = {
  printPreset: PrintPreset;
  includePageNumbers: boolean;
  chapterStartsNewPage: boolean;
};

export type BookExportRecord = {
  id: string;
  projectId: string;
  bookTitle: string;
  format: BookExportFormat;
  outputPath: string;
  targetLanguage: string;
  exportedAtUnixMs: number;
  profile: BookExportProfile;
};

export type BookSegment = {
  id: string;
  source: string;
  translation: string;
  status: SegmentStatus;
  qaNote: string | null;
  terms: string[];
};

export type BookChapter = {
  id: string;
  title: string;
  segments: BookSegment[];
};

export type BookProject = {
  id: string;
  sourcePath: string;
  title: string;
  format: BookFormat;
  sourceLanguage: string;
  targetLanguage: string;
  chapters: BookChapter[];
  publication: PublicationMetadata;
};

export function bookProgress(project: BookProject) {
  const segments = project.chapters.flatMap((chapter) => chapter.segments);
  if (!segments.length) return 0;
  const reviewed = segments.filter((segment) => segment.status === "reviewed").length;
  return Math.round((reviewed / segments.length) * 100);
}

export function bookIssueCount(project: BookProject) {
  return project.chapters.reduce(
    (count, chapter) => count + chapter.segments.filter((segment) => segment.status === "issue").length,
    0,
  );
}
