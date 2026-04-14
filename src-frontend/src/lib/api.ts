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
