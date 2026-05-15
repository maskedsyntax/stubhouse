import {
  activateEnv,
  clearHistory,
  deactivateEnv,
  importPostman,
  listEnvs,
  listHistory,
  listRequests,
  loadHistory,
  loadRequest,
  openWorkspace,
  saveRequest,
  type ActiveEnvironment,
  type Compose,
  type EnvironmentEntry,
  type HistoryEntry,
  type HistoryReplay,
  type ImportSummary,
  type RequestDefinition,
  type RequestEntry,
  type WorkspaceInfo,
} from "./api";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

type State = {
  info: WorkspaceInfo | null;
  entries: RequestEntry[];
  history: HistoryEntry[];
  envs: EnvironmentEntry[];
  activeEnv: ActiveEnvironment | null;
  activeId: string | null;
  error: string | null;
};

function createStore() {
  let state = $state<State>({
    info: null,
    entries: [],
    history: [],
    envs: [],
    activeEnv: null,
    activeId: null,
    error: null,
  });

  async function pickAndOpen(): Promise<boolean> {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected !== "string") return false;
    return await openPath(selected);
  }

  async function openPath(path: string): Promise<boolean> {
    try {
      const info = await openWorkspace(path);
      state.info = info;
      state.error = null;
      await refresh();
      return true;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
      return false;
    }
  }

  async function refresh(): Promise<void> {
    if (!state.info) return;
    try {
      state.entries = await listRequests();
      state.history = await listHistory(100);
      state.envs = await listEnvs();
      state.activeEnv = null;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
    }
  }

  async function activate(name: string): Promise<void> {
    if (!state.info) return;
    try {
      state.activeEnv = await activateEnv(name);
      state.error = null;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
    }
  }

  async function deactivate(): Promise<void> {
    if (!state.info) return;
    try {
      await deactivateEnv();
      state.activeEnv = null;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
    }
  }

  async function pickAndImportPostman(): Promise<ImportSummary | null> {
    if (!state.info) {
      state.error = "Open a workspace first.";
      return null;
    }
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Postman Collection", extensions: ["json"] }],
    });
    if (typeof selected !== "string") return null;
    try {
      const summary = await importPostman(selected);
      state.error = null;
      await refresh();
      return summary;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
      return null;
    }
  }

  async function refreshHistory(): Promise<void> {
    if (!state.info) return;
    try {
      state.history = await listHistory(100);
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
    }
  }

  async function replayHistory(id: number): Promise<HistoryReplay | null> {
    try {
      return await loadHistory(id);
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
      return null;
    }
  }

  async function wipeHistory(): Promise<void> {
    if (!state.info) return;
    try {
      await clearHistory();
      state.history = [];
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
    }
  }

  async function load(id: string): Promise<RequestDefinition | null> {
    try {
      const def = await loadRequest(id);
      state.activeId = id;
      state.error = null;
      return def;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
      return null;
    }
  }

  async function save(
    collection: string,
    slug: string,
    name: string,
    description: string,
    compose: Compose,
  ): Promise<string | null> {
    try {
      const def: RequestDefinition = { name, description, ...compose };
      const id = await saveRequest(collection, slug, def);
      state.activeId = id;
      state.error = null;
      await refresh();
      return id;
    } catch (e) {
      state.error = typeof e === "string" ? e : String(e);
      return null;
    }
  }

  return {
    get info() { return state.info; },
    get entries() { return state.entries; },
    get history() { return state.history; },
    get envs() { return state.envs; },
    get activeEnv() { return state.activeEnv; },
    get activeId() { return state.activeId; },
    get error() { return state.error; },
    set activeId(v: string | null) { state.activeId = v; },
    pickAndOpen,
    openPath,
    refresh,
    refreshHistory,
    replayHistory,
    wipeHistory,
    activate,
    deactivate,
    pickAndImportPostman,
    load,
    save,
  };
}

export const workspace = createStore();
export type WorkspaceStore = ReturnType<typeof createStore>;
