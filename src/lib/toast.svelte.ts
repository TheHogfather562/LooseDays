// Tiny app-wide toast layer. Components (and the `api` wrapper) call
// `toast.success(...)` / `toast.error(...)` to surface the outcome of an
// action; `Toast.svelte`, mounted once in `AppShell`, renders whatever is in
// `toast.items`. There's no other feedback channel in the app, so a swallowed
// promise rejection used to be completely invisible — this is what makes those
// failures show up.

export type ToastKind = 'success' | 'error';

export interface ToastItem {
	id: number;
	kind: ToastKind;
	message: string;
}

let nextId = 0;

export const toast = $state({ items: [] as ToastItem[] });

function push(kind: ToastKind, message: string) {
	const id = nextId++;
	toast.items.push({ id, kind, message });
	// Errors linger a little longer than confirmations — they usually mean the
	// user needs to retry, so give them time to read it.
	const ttl = kind === 'error' ? 5000 : 3000;
	setTimeout(() => dismiss(id), ttl);
	return id;
}

export function dismiss(id: number) {
	const i = toast.items.findIndex((t) => t.id === id);
	if (i !== -1) toast.items.splice(i, 1);
}

export function toastSuccess(message: string) {
	return push('success', message);
}

export function toastError(message: string) {
	return push('error', message);
}

/** Turns an unknown thrown value into a human-readable message. `api.ts`
 * throws real `Error`s with the backend's message, so prefer that. */
export function errorMessage(err: unknown, fallback = 'Something went wrong — try again.'): string {
	if (err instanceof Error && err.message) return err.message;
	return fallback;
}

/** Run a mutating action, surfacing any failure as an error toast (and an
 * optional success toast). Returns whether it succeeded, so callers can gate
 * follow-up UI (navigation, "sent" state) on the real outcome instead of
 * optimistically assuming it worked. */
export async function withToast(
	action: () => Promise<unknown>,
	opts: { success?: string; error?: string } = {}
): Promise<boolean> {
	try {
		await action();
		if (opts.success) toastSuccess(opts.success);
		return true;
	} catch (err) {
		console.error(err);
		toastError(errorMessage(err, opts.error));
		return false;
	}
}
