<script lang="ts">
import { generate_password } from '../../wasm/public/package.js'
import { EMPTY } from './util.js';

let domain = $state(EMPTY);

let password = $state(EMPTY);

let generatedPassword = $state(EMPTY);

function mask(password: String) {
	return '*'.repeat(password.length);
}

function generate(ev: Event) {
	ev.preventDefault();
	if (domain == EMPTY || password == EMPTY) {
		console.info('empty domain or password');
	} else {
		generatedPassword = generate_password(domain, password);
	}
}

function keydown(ev: KeyboardEvent) {
	if (ev.code == 'Escape') {
		ev.preventDefault();
		const id = (ev.target as Element).getAttribute('id');
		if (id === 'domain-password') {
			password = EMPTY;
		} else if (id == 'domain') {
			domain = EMPTY;
		}

		console.info('clearing password');
		clearGenerated()
	}
}

export function clearGenerated() {
	generatedPassword = EMPTY;
}

function clearCurrent() {
	generatedPassword = EMPTY;
	password = EMPTY;
}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<form onsubmit={generate} onkeydown={keydown}>
	<div class="mb-3">
		<label for="domain" class="form-label">Domain</label>
		<!-- svelte-ignore a11y_autofocus -->
		<input type="text" class="form-control" id="domain" bind:value={domain} onkeydown={keydown} autofocus />
	</div>
	<div class="mb-3">
		<label for="domain-password" class="form-label">Password</label>
		<input type="password" class="form-control" id="domain-password" bind:value={password} onkeydown={keydown} oninput={clearGenerated} />
	</div>
	<button type="submit" class="btn btn-primary">Submit</button>
	<!-- <button type="button" class="btn btn-secondary" onclick={clearGenerated}>Clear</button> -->

	{#if generatedPassword !== EMPTY}
	<div class="alert alert-light" role="alert">
		Generated password: {mask(generatedPassword)}
		<br>
		{generatedPassword}
	</div>
	{/if}
</form>
