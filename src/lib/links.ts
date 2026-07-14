export function smsLink(phone: string, body: string): string {
	return `sms:${phone.replace(/\s/g, '')}?&body=${encodeURIComponent(body)}`;
}

export function whatsappLink(phone: string, body: string): string {
	return `https://wa.me/${phone.replace(/\D/g, '')}?text=${encodeURIComponent(body)}`;
}
