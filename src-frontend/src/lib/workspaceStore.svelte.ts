import {
  listRequests,
  loadRequest,
  openWorkspace,
  saveRequest,
  type Compose,
  type RequestDefinition,
  type RequestEntry,
  type WorkspaceInfo,
} from "./api";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

type State = {
  info: WorkspaceInfo | null;
  entries: RequestEntry[];
  activeId: string | null;
  error: string | null;
};

function createStore() {
  let state = $state<State>({
    info: null,
    entries: [],
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
    get activeId() { return state.activeId; },
    get error() { return state.error; },
    pickAndOpen,
    openPath,
    refresh,
    load,
    save,
  };
}

export const workspace = createStore();
export type WorkspaceStore = ReturnType<typeof createStore>;
