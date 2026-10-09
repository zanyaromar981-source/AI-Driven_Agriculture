// Talks to the Jutyar backend (FRONTEND.md). One place for the address, the token, errors and ETags.
// In development /v1 goes through the Vite proxy; a built site calls VITE_API_URL (must be allowed by
// the server's CORS list).

export const API_BASE: string = (import.meta.env.VITE_API_URL as string | undefined)?.replace(/\/$/, '') || '/v1';

/** An answer that was not 2xx. `code` is the server's `error` field (FRONTEND.md section 13). */
export class ApiError extends Error {
  constructor(readonly status: number, readonly code: string, readonly detail: string, readonly field?: string, readonly retryAfter?: number) {
    super(`${status} ${code}: ${detail}`);
  }
}

let token: string | null = null;
let onUnauthorized: (() => void) | null = null;
let onApiVersion: ((v: string) => void) | null = null;
export const setToken = (t: string | null) => { token = t; };
export const getToken = () => token;
export const setUnauthorizedHandler = (f: () => void) => { onUnauthorized = f; };
export const setApiVersionHandler = (f: (v: string) => void) => { onApiVersion = f; };

export interface RawAnswer<T> { status: number; data: T | null; etag: string | null }

/** One request. `etag` turns a GET into a conditional GET: a 304 answers `data: null`. */
export async function raw<T>(method: string, path: string, opts: { body?: unknown; etag?: string | null; auth?: boolean; signal?: AbortSignal; idempotencyKey?: string } = {}): Promise<RawAnswer<T>> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (opts.body !== undefined) headers['Content-Type'] = 'application/json';
  if (opts.etag) headers['If-None-Match'] = opts.etag;
  if (opts.idempotencyKey) headers['Idempotency-Key'] = opts.idempotencyKey;
  const sentWith = opts.auth !== false ? token : null;
  if (sentWith) headers.Authorization = 'Bearer ' + sentWith;
  let res: Response;
  try {
    // no-store: our own cache (src/api/cache.ts) owns ETags; the browser's HTTP cache would revalidate the
    // same answer a second time and Chrome then cancels one of the two requests.
    res = await fetch(API_BASE + path, { method, headers, body: opts.body === undefined ? undefined : JSON.stringify(opts.body), signal: opts.signal, cache: 'no-store' });
  } catch (e) {
    if ((e as Error).name === 'AbortError') throw e;
    throw new ApiError(0, 'offline', 'The server could not be reached');
  }
  const v = res.headers.get('X-Api-Version');
  if (v && onApiVersion) onApiVersion(v);
  if (res.status === 304) return { status: 304, data: null, etag: opts.etag ?? null };
  if (res.status === 204) return { status: 204, data: null, etag: null };
  let body: unknown = null;
  const text = await res.text();
  if (text) { try { body = JSON.parse(text); } catch { body = null; } }
  if (!res.ok) {
    const b = (body ?? {}) as { error?: string; detail?: string; field?: string; retry_after_s?: number };
    // only the session that sent this request may be signed out by its 401 (a late answer from an old
    // token must not end the next session)
    if (res.status === 401 && sentWith && sentWith === token && onUnauthorized) onUnauthorized();
    throw new ApiError(res.status, b.error ?? 'server_error', b.detail ?? res.statusText, b.field, b.retry_after_s);
  }
  return { status: res.status, data: body as T, etag: res.headers.get('ETag') };
}

export const api = {
  get: <T>(path: string) => raw<T>('GET', path).then(r => r.data as T),
  post: <T>(path: string, body?: unknown, idempotencyKey?: string) => raw<T>('POST', path, { body: body ?? {}, idempotencyKey }).then(r => r.data as T),
  put: <T>(path: string, body: unknown) => raw<T>('PUT', path, { body }).then(r => r.data as T),
  del: (path: string) => raw<null>('DELETE', path).then(() => undefined),
};

/** Query string from an object; empty values are left out (an empty `page=` is a 400, FRONTEND.md 12). */
export function qs(params: Record<string, string | number | boolean | null | undefined>): string {
  const p = Object.entries(params).filter(([, v]) => v !== undefined && v !== null && v !== '');
  return p.length ? '?' + p.map(([k, v]) => encodeURIComponent(k) + '=' + encodeURIComponent(String(v))).join('&') : '';
}

/** Photos behind the token (inbox): fetch the bytes and make a local address for <img>. */
export async function authedBlobUrl(path: string): Promise<string> {
  const res = await fetch(API_BASE + path, { headers: token ? { Authorization: 'Bearer ' + token } : {} });
  if (!res.ok) throw new ApiError(res.status, 'photo_failed', res.statusText);
  return URL.createObjectURL(await res.blob());
}
