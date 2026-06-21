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
}

export const EMPTY_USER_STATE: UserState = {
	user: EMPTY,
	password: EMPTY,
	loginTask: null,
	authenticateTask: null,
}
