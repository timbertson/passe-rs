import { Config } from '../../wasm/public/package.js'
import { User, UserState } from './Authentication.js';
import { fetchReq, notNull } from './util.js';
import { Authentication } from './Authentication.js';

const CACHE_KEY = 'user-db';

export class Db {
	config: Config;
	userState: UserState;

	static loadCached(userState: UserState): Db {
		const cachedStr = window.localStorage.getItem(CACHE_KEY);
		let config = null;
		try {
			config = Config.new(cachedStr || undefined);
		} catch(e) {
			console.error("Error loading cached DB:", e);
			config = Config.new(undefined);
		}
		return new Db(notNull(config), userState);
	}

	constructor(config: Config, userState: UserState) {
		this.config = config;
		this.userState = userState;
	}
	
	save() {
		window.localStorage.setItem(CACHE_KEY, this.config.serialize());
		this.config.update_after_save();
	}
	
	generatePassword(domain: string, password: string): string {
		return this.config.generate_password(domain, password)
	}
	
	tryAuthenticate = () => {
		const req = this.config.authenticate_request();
		if (req) {
			console.info(`Attempting to re-authenticate cached user`)
			this.userState.authenticateTask = (async () => {
				const user = await fetchReq<string>(req);
				this.userState.loginTask = Promise.resolve(user);
				return user;
			})();
		}
	}

	private login_or_register = async (req: Request): Promise<User> => {
		const response = await fetchReq<Authentication>(req);
		console.log("Logged in successfully");
		return this.update_after_login(response);
	}
	
	private update_after_login(auth: Authentication) {
		this.config.update_auth(auth);
		this.save();
		return auth.user;
	}

	submitLogin = (ev: Event) => {
		ev.preventDefault();
		this.userState.loginTask = (async (): Promise<User> => {
			return this.login_or_register(this.config.login_request(this.userState.user, this.userState.password));
		})();
	}
	
	clearLoginTask = () => {
		this.userState.loginTask = null;
	}

	submitRegister = (ev: Event) => {
		ev.preventDefault();
		this.userState.loginTask = (async (): Promise<User> => {
			return this.login_or_register(this.config.register_request(this.userState.user, this.userState.password));
		})();
	}
	
	sync = async () => {
		this.userState.syncTask = (async () => {
			const req = this.config.sync_request();
			const newDb = await fetchReq<Object>(req);
			console.log("sync completed");
			this.config.set_db(newDb);
			this.save();
			this.userState.invalidateDb += 1;
			this.userState.syncState = 'in-sync';
		})();
	}
}
