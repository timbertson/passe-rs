<script lang="ts">
import init from '../../wasm/public/package.js'
import { Db } from './Db';
import PasswordForm from './PasswordForm.svelte';
import UserPanel from './UserPanel.svelte';

async function load(): Promise<Db> {
	let wasm = await init({ module_or_path: '/wasm/public/package_bg.wasm' })
	console.log("Loaded!: ", wasm);
	return Db.loadCached();
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
		<PasswordForm {db}/>
	</div>
{:catch e}
	<div class="container">
		<h1>Error:</h1>
		<pre>{e}</pre>
	</div>
{/await}
