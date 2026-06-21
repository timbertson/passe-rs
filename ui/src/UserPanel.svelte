<script lang="ts">
import { Db } from './Db';
import SyncWidget from './SyncWidget.svelte';
import UserForm from './UserForm.svelte';

const { db }: { db: Db } = $props();
</script>

<header class="shadow-lg border-bottom mb-4 py-3 d-flex bg-light-subtle">
	<div class="container">
		{#if db.userState.loginTask != null}
			{#await db.userState.loginTask}
				(spin spin...)
			{:then user}
				<div class="row">
					<div class="col">
						<span class="fs-4">passe</span>
					</div>
					<div class="col">
						<span>Logged in: {user}</span>
						{#await db.userState.syncTask}
							:syncing...:
						{:then}
							:not-syncing:
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
					<button type="button" class="btn-close float-end" onclick={db.clearLoginTask} aria-label="Close"></button>
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
