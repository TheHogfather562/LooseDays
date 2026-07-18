// Thin wrapper around the browser WebAuthn API. The backend (see
// backend/src/handlers/passkeys.rs) speaks the standard base64url-encoded
// JSON shape for creation/request options and credential responses, so
// everything here is just converting to/from ArrayBuffers around that.

import { api } from './api';

export function passkeysSupported(): boolean {
	return typeof window !== 'undefined' && typeof window.PublicKeyCredential !== 'undefined';
}

interface CredentialDescriptorJSON {
	id: string;
	type: 'public-key';
	transports?: AuthenticatorTransport[];
}

/** The JSON-serialised shape webauthn-rs sends for registration options —
 * identical to the browser's own `PublicKeyCredentialCreationOptions`,
 * except `challenge`/credential `id`s are base64url strings instead of
 * ArrayBuffers. */
export interface CreationOptionsJSON extends Record<string, unknown> {
	rp: { id?: string; name: string };
	user: { id: string; name: string; displayName: string };
	challenge: string;
	pubKeyCredParams: PublicKeyCredentialParameters[];
	excludeCredentials?: CredentialDescriptorJSON[];
}

/** Same idea for sign-in (`PublicKeyCredentialRequestOptions`). */
export interface RequestOptionsJSON extends Record<string, unknown> {
	challenge: string;
	rpId: string;
	allowCredentials?: CredentialDescriptorJSON[];
}

function bufferToBase64Url(buffer: ArrayBuffer): string {
	let binary = '';
	for (const byte of new Uint8Array(buffer)) binary += String.fromCharCode(byte);
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function base64UrlToBuffer(value: string): ArrayBuffer {
	const base64 = value.replace(/-/g, '+').replace(/_/g, '/');
	const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=');
	const binary = atob(padded);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes.buffer;
}

function toCreationOptions(publicKey: CreationOptionsJSON): CredentialCreationOptions {
	return {
		publicKey: {
			...publicKey,
			challenge: base64UrlToBuffer(publicKey.challenge),
			user: { ...publicKey.user, id: base64UrlToBuffer(publicKey.user.id) },
			excludeCredentials: (publicKey.excludeCredentials ?? []).map((c) => ({
				...c,
				id: base64UrlToBuffer(c.id)
			}))
		}
	};
}

function toRequestOptions(publicKey: RequestOptionsJSON): CredentialRequestOptions {
	return {
		publicKey: {
			...publicKey,
			challenge: base64UrlToBuffer(publicKey.challenge),
			allowCredentials: (publicKey.allowCredentials ?? []).map((c) => ({
				...c,
				id: base64UrlToBuffer(c.id)
			}))
		}
	};
}

function registerCredentialToJSON(cred: PublicKeyCredential) {
	const response = cred.response as AuthenticatorAttestationResponse;
	return {
		id: cred.id,
		rawId: bufferToBase64Url(cred.rawId),
		type: cred.type,
		response: {
			attestationObject: bufferToBase64Url(response.attestationObject),
			clientDataJSON: bufferToBase64Url(response.clientDataJSON),
			transports: response.getTransports?.() ?? []
		},
		extensions: cred.getClientExtensionResults()
	};
}

function authCredentialToJSON(cred: PublicKeyCredential) {
	const response = cred.response as AuthenticatorAssertionResponse;
	return {
		id: cred.id,
		rawId: bufferToBase64Url(cred.rawId),
		type: cred.type,
		response: {
			authenticatorData: bufferToBase64Url(response.authenticatorData),
			clientDataJSON: bufferToBase64Url(response.clientDataJSON),
			signature: bufferToBase64Url(response.signature),
			userHandle: response.userHandle ? bufferToBase64Url(response.userHandle) : null
		},
		extensions: cred.getClientExtensionResults()
	};
}

/** Registers a new passkey for the signed-in user. Must be called from
 * within a user gesture (e.g. a button click). */
export async function registerPasskey(label?: string): Promise<void> {
	const start = await api.passkeyRegisterStart();
	const credential = await navigator.credentials.create(toCreationOptions(start.publicKey));
	if (!(credential instanceof PublicKeyCredential)) {
		throw new Error('Passkey registration was cancelled');
	}
	await api.passkeyRegisterFinish(start.challengeId, registerCredentialToJSON(credential), label);
}

/** Discoverable (usernameless) sign-in: the browser's own credential picker
 * identifies the account, so no email needs to be entered first. */
export async function signInWithPasskey(): Promise<{ redirect: string }> {
	const start = await api.passkeyAuthStart();
	const credential = await navigator.credentials.get(toRequestOptions(start.publicKey));
	if (!(credential instanceof PublicKeyCredential)) {
		throw new Error('Passkey sign-in was cancelled');
	}
	return api.passkeyAuthFinish(start.challengeId, authCredentialToJSON(credential));
}
