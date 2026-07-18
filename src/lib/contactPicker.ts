// Thin wrapper around the browser Contact Picker API (Chrome/Android only,
// requires a secure context + user gesture) so people can pull a contact's
// name + number straight from their phone instead of typing it in.

interface ContactsManager {
	select(properties: string[], options?: { multiple?: boolean }): Promise<ContactPickerResult[]>;
}

interface ContactPickerResult {
	name?: string[];
	tel?: string[];
}

export function contactPickerSupported(): boolean {
	return typeof navigator !== 'undefined' && 'contacts' in navigator && 'ContactsManager' in window;
}

export interface PickedContact {
	name: string;
	phone: string;
}

export async function pickContacts(): Promise<PickedContact[]> {
	const contactsApi = (navigator as unknown as { contacts: ContactsManager }).contacts;
	const results = await contactsApi.select(['name', 'tel'], { multiple: true });
	const picked: PickedContact[] = [];
	for (const r of results) {
		const phone = r.tel?.[0];
		if (!phone) continue;
		picked.push({ name: r.name?.[0] || phone, phone });
	}
	return picked;
}
