export type TranslationItem = { id: string; source: string; target: string; speaker: string | null; sourceFile: string; qa: "passed" | "warning" | "blocking" };
export type TranslationRun = { items: TranslationItem[]; warningFindings: number; blockingFindings: number; failedSegmentIds: string[] };
export type TranslationProgressState = { phase: "idle" | "extracting" | "translating" | "qa" | "completed" | "failed"; completed: number; total: number; failed: number; warningFindings: number; blockingFindings: number; message: string; concurrency?: number; throughput?: number; etaSeconds?: number };
export type TranslationProgressEvent = TranslationProgressState & { runId: string };
export type ResumableTask = { id: string; projectPath: string; state: "pending" | "running" | "paused"; total: number; completed: number; failed: number; updatedAtUnixMs: number };
