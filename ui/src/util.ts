import type { Authentication } from './State.js';

export function notNull<A>(obj: A|null|undefined): A {
	if (obj == null) {
		throw Error("Unexpected null");
	}
	return obj;
}

export const EMPTY = '';

export class HttpError extends Error {
	response: Response;

	constructor(response: Response, message: string) {
		super(message);
		this.response = response;
	}
}

export async function fetchReq<T>(req: Request): Promise<T> {
	const response = await fetch(req);
	if (!response.ok) {
		const contentType: String|null = response.headers.get('content-type');
		console.log("Failed response:", contentType);
		if (contentType === 'application/json') {
			const responseJson = await response.json();
			throw new HttpError(response, responseJson.message || `Request failed: ${req.url}`);
		} else {
			throw new HttpError(response, await response.text());
		}
	} else {
		const body: T = await response.json();
		return body;
	}
}

export async function postAPI<T>(url: string, auth: null|Authentication, data: Object|null): Promise<T> {
	const headers = new Headers();
	if (auth) {
		headers.append('Authorization', JSON.stringify(auth));
	}
	const response = await fetch(url, {
		method: 'POST',
		body: data ? JSON.stringify(data) : null,
		headers,
	});
	if (!response.ok) {
		const contentType: String|null = response.headers.get('content-type');
		console.log("Failed response:", contentType);
		if (contentType === 'application/json') {
			const responseJson = await response.json();
			throw new HttpError(response, responseJson.message || `Request failed: ${url}`);
		} else {
			throw new HttpError(response, await response.text());
		}
	} else {
		const body: T = await response.json();
		return body;
	}
}
