<script>
import init, { hello } from '../../web/public/package.js'

async function load() {
	let wasm = await init('/web/public/package_bg.wasm')
	console.log("Loaded!: ", wasm);
	let response = hello();
	console.log(response);
}

let initialize = $state(load().catch((e) => {
	console.error("Error loading: ", e);
	throw e;
}));
</script>


{#await initialize}
	<h1>Loading...</h1>
{:then}
	<h1>Ready!</h1>
{:catch e}
	<h1>Error:</h1>
	<pre>{e}</pre>
{/await}
