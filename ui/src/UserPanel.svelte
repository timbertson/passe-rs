<script lang="ts">
import { Db } from './Db';
import SyncWidget from './SyncWidget.svelte';
import UserForm from './UserForm.svelte';

const { db }: { db: Db } = $props();
function clearAuthentication(ev: Event) {
	ev.preventDefault();
	db.clearAuthentication();
}
</script>

<header class="shadow-lg border-bottom mb-4 py-3 d-flex">
	<div class="container">
		{#if db.userState.loginTask != null}
			{#await db.userState.loginTask}
				<div class="row">
					<div class="col">
						<span class="fs-4"><strong>passe</strong></span>
					</div>
					<div class="col text-center fs-4">
						<span>...</span>
					</div>
					<div class="col text-end">
					</div>
				</div>
			{:then user}
				<div class="row">
					<div class="col-md">
						<span class="fs-4"><strong>passe</strong></span>
					</div>
					<div class="col-md text-center fs-4">
						<span>{user}</span>
						<button class="btn btn-outline-secondary" onclick={clearAuthentication}>x</button>
					</div>
					<div class="col-md text-end">
						{#await db.userState.syncTask}
							...
						{:then}
							<SyncWidget {db}/>
						{:catch e}
							:error -- {e}:
							<SyncWidget {db}/>
						{/await}
					</div>
				</div>
			{:catch e}
				<UserForm {db} />
				<div class="alert alert-danger mt-3 mb-0">
					Error:
					{#if e instanceof Error}
						{e.message}
					{:else}
						{e}
					{/if}
					<button tabindex="-1" type="button" class="btn-close float-end" onclick={db.clearLoginTask} aria-label="Close"></button>
				</div>
			{/await}
		{:else} <!-- loginTask == null -->
			{#if db.userState.authenticateTask != null}
				{#await db.userState.authenticateTask}
					(loading...)
				{/await}
			{/if}
			<UserForm {db} />
		{/if}
	</div>
</header>
