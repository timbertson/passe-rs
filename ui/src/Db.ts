import { Config } from '../../wasm/public/package.js'

const CACHE_KEY = 'user-db';

const localStorage = window.localStorage;

function setCached(value: null|String) {
	if (value == null) {
		localStorage.removeItem(CACHE_KEY);
	} else {
		localStorage.setItem(CACHE_KEY, JSON.stringify(value));
	}
}

export class Db {
	config: Config;

	static loadCached(): Db {
		const cachedStr = window.localStorage.getItem(CACHE_KEY);
		try {
			return new Db(Config.new(cachedStr || undefined))
		} catch(e) {
			console.error("Error loading cached DB:", e);
			return new Db(Config.new(undefined))
		}
	}

	constructor(config: Config) {
		this.config = config;
	}
	
	generatePassword(domain: string, password: string): string {
		return this.config.generate_password(domain, password)
	}
	
	tryAuthenticate() {
		const req = this.config.authenticate_request();
	}
}
