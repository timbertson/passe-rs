<script lang="ts">
import { Db } from './Db';
import UserForm from './UserForm.svelte';
import { UserCtl, loadCached } from './User';
import type { UserState } from './User';
import { EMPTY, notNull } from './util.js';
import { onMount } from 'svelte';

const { db }: { db: Db } = $props();

const userState: UserState = $state({
	user: EMPTY,
	password: EMPTY,
	loginTask: null,
	authenticateTask: null,
});

onMount(() => {
	userState.authenticateTask = db.tryAuthenticate();
});

</script>

<header class="shadow-lg border-bottom mb-4 py-3 d-flex bg-light-subtle">
	<div class="container">
		{#await userState.loginTask}
			(spin spin...)
		{:then loginResponse}
			{#if loginResponse == null}
				{#if ctl.hasCachedAuth()}
					{#await userState.authenticateTask}
						(loading...)
					{/await}
				{/if}
				<UserForm {ctl} />
			{:else}
				<div class="row">
					<div class="col">
						<span class="fs-4">passe</span>
					</div>
					<div class="col">
						<span>Logged in: {loginResponse.user}</span>
					</div>
				</div>
			{/if}
		{:catch e}
			<UserForm {ctl} />
			<div class="alert alert-danger mt-3 mb-0">
				Error:
				{#if e instanceof Error}
					{e.message}
				{:else}
					{e}
				{/if}
				<button type="button" class="btn-close float-end" onclick={userState.clearLogin} aria-label="Close"></button>
			</div>
		{/await}
	</div>
</header>
