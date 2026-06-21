import { EMPTY, HttpError, notNull, postAPI } from './util.js';
import type { Authentication } from './Authentication.js';

const CACHED_AUTH = 'cached-auth';


const localStorage = window.localStorage;

function setCached(value: null|Authentication) {
	if (value == null) {
		localStorage.removeItem(CACHED_AUTH);
	} else {
		localStorage.setItem(CACHED_AUTH, JSON.stringify(value));
	}
}

export function loadCached() {
	const cachedStr = window.localStorage.getItem(CACHED_AUTH);
	if (cachedStr != null) {
		try {
			const cached = JSON.parse(cachedStr) as Authentication;
			return {
				user: notNull(cached.user),
				token: cached.token,
			};
		} catch(e) {
			console.error("Error loading cached auth:", e);
		}
	}
	return null;
}


export type UserState = {
	user: String,
	password: String,
	loginTask: null | Promise<Authentication>,
	authenticateTask: null | Promise<Authentication>,
}

export class UserCtl {
	state: UserState;
	
	constructor(state: UserState) {
		this.state = state;
	}
	
	hasCachedAuth = () => {
		return this.state.cachedAuth != null;
	}

	submitLogin = (ev: Event) => {
		ev.preventDefault();
		this.state.loginTask = this.login();
	}

	private userPayload = () => ({ user: this.state.user, password: this.state.password });

	login = async (): Promise<Authentication> => {
		const response = await postAPI<Authentication>('/login', null, this.userPayload());
		console.log("Logged in successfully");
		setCached(response);
		return response;
	}

	submitRegister = (ev: Event) => {
		ev.preventDefault();
		this.state.loginTask = this.register();
	}

	clearLogin = () => {
		this.state.loginTask = Promise.resolve(null);
	}
	
	tryAuthenticate = () => {
		if (this.state.cachedAuth != null) {
			this.state.authenticateTask = this.authenticate(this.state.cachedAuth);
		}
	}

	register = async (): Promise<Authentication> => {
		const response = await postAPI<Authentication>('/register', null, this.userPayload());
		console.log("Registered successfully");
		setCached(response);
		return response;
	}

	authenticate = async (cached: Authentication): Promise<null|Authentication> => {
		try {
			const response = await postAPI<Authentication>('/authenticate', cached, null);
			console.log("Authenticated successfully");
			
			// if auth succeeds, it counts as a login
			this.state.loginTask = Promise.resolve(cached);
			return response;
		} catch(e) {
			console.warn("Authenticate error:", e);
			if (e instanceof HttpError && e.response.status === 403) {
				console.info("Unauthorized; clearing cached token");
				setCached(null);
				return null;
			}
			return null;
		}
	}
}
