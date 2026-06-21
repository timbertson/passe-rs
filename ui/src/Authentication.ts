import { EMPTY } from "./util"

export type Authentication = {
	user: string,
	token: string,
}

export type User = string;

type SyncState = null | 'stale' | 'in-sync'

export type UserState = {
	user: string,
	password: string,
	loginTask: null | Promise<User>,
	authenticateTask: null | Promise<User>,
	syncState: SyncState,
	syncTask: Promise<void>
	invalidateDb: number,
}

export const EMPTY_USER_STATE: UserState = {
	user: EMPTY,
	password: EMPTY,
	loginTask: null,
	authenticateTask: null,
	syncState: null,
	syncTask: Promise.resolve(),
	invalidateDb: 0,
}
