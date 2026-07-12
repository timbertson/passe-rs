<script lang="ts">
import init from '../../wasm/public/package.js'
import { EMPTY_USER_STATE } from './State.js';
import { Db } from './Db';
import DomainConfig from './DomainConfig.svelte';
import PasswordForm from './PasswordForm.svelte';
import UserPanel from './UserPanel.svelte';

async function load(): Promise<Db> {
	await init({ module_or_path: '/wasm/public/package_bg.wasm' });
	const userState = $state(EMPTY_USER_STATE);
	const db = Db.loadCached(userState);
	db.tryAuthenticate(); // kick off but don't wait
	(window as any).passeDb = db;
	return db;
}

let initialize = $state(load().catch((e) => {
	console.error("Error loading: ", e);
	throw e;
}));

// notNull(document.getElementsByTagName('body')).addEventListener('click');
</script>

{#await initialize}
	<div class="container">
		<h1 class="text-center mt-5" style="color: #ffffff88;">Loading WASM...</h1>
	</div>
{:then db}
	<UserPanel {db}/>
	<div class="container mb-5">
		<div class="row">
			<div class="col-xl gy-2">
				<PasswordForm {db}/>
			</div>
			<div class="col-xl-5 gy-4">
				<DomainConfig {db}/>
			</div>
		</div>
		{#if db.userState.toastMessage != null}
			<div class="toast show p-3 text-bg-primary border-0">{db.userState.toastMessage}</div>
		{/if}
	</div>
{:catch e}
	<div class="container">
		<h1>Error:</h1>
		<pre>{e}</pre>
	</div>
{/await}
