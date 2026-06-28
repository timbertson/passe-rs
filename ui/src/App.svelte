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
	db.tryAuthenticate();
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
		<h1>Loading WASM...</h1>
	</div>
{:then db}
	<UserPanel {db}/>
	<div class="container">
		<div class="row">
			<div class="col-xl gy-2">
				<PasswordForm {db}/>
			</div>
			<div class="col-xl-5 gy-4">
				<DomainConfig {db}/>
			</div>
		</div>
	</div>
{:catch e}
	<div class="container">
		<h1>Error:</h1>
		<pre>{e}</pre>
	</div>
{/await}
