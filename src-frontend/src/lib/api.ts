import { invoke } from "@tauri-apps/api/core";

export type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS";

export interface RequestDto {
  method: Method;
  url: string;
  headers: Array<[string, string]>;
  body: string | null;
}

export interface ResponseDto {
  status: number;
  headers: Array<[string, string]>;
  body: string;
  elapsed_ms: number;
  size_bytes: number;
}

export async function sendRequest(req: RequestDto): Promise<ResponseDto> {
  return await invoke<ResponseDto>("send_request", { req });
}
