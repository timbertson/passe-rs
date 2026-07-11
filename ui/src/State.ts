import { DEFAULT_DOMAIN_CONFIG, DomainConfig } from "./Db";
import { EMPTY } from "./util"

export type Authentication = {
	user: string,
	token: string,
}

export type User = string;

export type UserState = {
	user: string,
	password: string,
	loginTask: null | Promise<User>,
	authenticateTask: null | Promise<User>,
	syncTask: Promise<void>
	invalidateDb: number,
	domain: string,
	domainConfig: DomainConfig,
	toastMessage: string|null,
}

export const EMPTY_USER_STATE: UserState = {
	// login
	user: EMPTY,
	password: EMPTY,
	loginTask: null,
	authenticateTask: null,
	
	// DB
	syncTask: Promise.resolve(),
	invalidateDb: 0,

	// domain config
	domain: EMPTY,
	domainConfig: { ... DEFAULT_DOMAIN_CONFIG },
	
	// UI
	toastMessage: null,
}
