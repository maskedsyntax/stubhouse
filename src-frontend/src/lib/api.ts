import { invoke } from "@tauri-apps/api/core";

export type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export type Auth =
  | { kind: "none" }
  | { kind: "bearer"; token: string }
  | { kind: "basic"; username: string; password: string }
  | { kind: "apikey"; in: "header" | "query"; name: string; value: string };

export type Body =
  | { kind: "none" }
  | { kind: "json"; text: string }
  | { kind: "text"; content_type: string; text: string }
  | { kind: "form"; fields: Array<[string, string]> };

export interface Compose {
  method: Method;
  url: string;
  query: Array<[string, string]>;
  headers: Array<[string, string]>;
  auth: Auth;
  body: Body;
}

export interface ResponseDto {
  status: number;
  headers: Array<[string, string]>;
  body: string;
  elapsed_ms: number;
  size_bytes: number;
  history_id: number | null;
}

export interface HistoryEntry {
  id: number;
  ts: number;
  method: string;
  url: string;
  status: number;
  elapsed_ms: number;
  size_bytes: number;
}

export interface HistoryReplay {
  request: Compose;
  response: ResponseDto;
}

export interface WorkspaceManifest {
  name: string;
  version: string;
}

export interface WorkspaceInfo {
  root: string;
  manifest: WorkspaceManifest;
}

export interface RequestEntry {
  id: string;
  name: string;
  collection: string;
}

export interface RequestDefinition extends Compose {
  name: string;
  description: string;
}

export async function sendRequest(req: Compose): Promise<ResponseDto> {
  return await invoke<ResponseDto>("send_request", { req });
}

export async function openWorkspace(path: string): Promise<WorkspaceInfo> {
  return await invoke<WorkspaceInfo>("open_workspace", { path });
}

export async function listRequests(): Promise<RequestEntry[]> {
  return await invoke<RequestEntry[]>("list_requests");
}

export async function loadRequest(id: string): Promise<RequestDefinition> {
  return await invoke<RequestDefinition>("load_request", { id });
}

export async function saveRequest(
  collection: string,
  slug: string,
  def: RequestDefinition,
): Promise<string> {
  return await invoke<string>("save_request", { collection, slug, def });
}

export async function listHistory(limit?: number): Promise<HistoryEntry[]> {
  return await invoke<HistoryEntry[]>("list_history", { limit: limit ?? null });
}

export async function loadHistory(id: number): Promise<HistoryReplay> {
  return await invoke<HistoryReplay>("load_history", { id });
}

export async function clearHistory(): Promise<number> {
  return await invoke<number>("clear_history");
}

export interface EnvironmentEntry {
  name: string;
  file: string;
}

export interface ActiveEnvironment {
  name: string;
  variables: Record<string, string>;
}

export async function listEnvs(): Promise<EnvironmentEntry[]> {
  return await invoke<EnvironmentEntry[]>("list_envs");
}

export async function activateEnv(name: string): Promise<ActiveEnvironment> {
  return await invoke<ActiveEnvironment>("activate_env", { name });
}

export async function deactivateEnv(): Promise<void> {
  await invoke<void>("deactivate_env");
}

export async function getActiveEnv(): Promise<ActiveEnvironment | null> {
  return await invoke<ActiveEnvironment | null>("active_env");
}

export interface ScenarioEntry {
  name: string;
  rules: number;
  active_rules: number;
}

export interface ScenarioActivation {
  scenario: string;
  files_changed: number;
  rules_changed: number;
}

export async function listMockScenarios(): Promise<ScenarioEntry[]> {
  return await invoke<ScenarioEntry[]>("list_mock_scenarios");
}

export async function activateMockScenario(name: string): Promise<ScenarioActivation> {
  return await invoke<ScenarioActivation>("activate_mock_scenario", { name });
}

export interface MockLog {
  method: string;
  path: string;
  matched_rule: string | null;
  status: number;
}

export interface MockServerStatus {
  running: boolean;
  bind: string;
  port: number;
  url: string;
  rules: number;
  logs: MockLog[];
}

export async function startMockServer(bind?: string, port?: number): Promise<MockServerStatus> {
  return await invoke<MockServerStatus>("start_mock_server", {
    bind: bind ?? null,
    port: port ?? null,
  });
}

export async function stopMockServer(): Promise<MockServerStatus> {
  return await invoke<MockServerStatus>("stop_mock_server");
}

export async function mockServerStatus(): Promise<MockServerStatus> {
  return await invoke<MockServerStatus>("mock_server_status");
}

export async function exportCurl(req: Compose): Promise<string> {
  return await invoke<string>("export_curl", { req });
}

export interface ImportSummary {
  imported: number;
  collections: string[];
}

export async function importPostman(path: string): Promise<ImportSummary> {
  return await invoke<ImportSummary>("import_postman", { path });
}
