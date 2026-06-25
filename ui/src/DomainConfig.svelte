<script lang="ts">
import { Db, domainConfigEq } from "./Db";
import type { DomainConfig } from "./Db";

let { db }: { db: Db } = $props();

let persistedOnly: null|DomainConfig = $derived(db.lookup(db.userState.domain));
let persisted = (): DomainConfig => persistedOnly || db.defaultConfig();

$effect(() => {
	db.userState.domainConfig = { ...persisted() };
	console.info("Reset domain form to match persisted");
})

let dirty = $derived.by(() => {
	return !domainConfigEq(persisted(), db.userState.domainConfig);
})
$inspect('domain:', db.userState.domain)
$inspect('persisted:', persistedOnly)
$inspect('domainConfig (form)', db.userState.domainConfig);

let canSave = () => {
	const domain = db.userState.domain;
	if (domain == '') {
		return false;
	}
	if (persistedOnly == null) {
		return true;
	}
	return dirty;
};

function submit(ev: Event) {
	ev.preventDefault();
	db.saveDomain(db.userState.domain, db.userState.domainConfig);
}

const buttonClass = () => canSave() ? "btn-primary" : "disabled btn-secondary";

</script>

<div class="card mt-5">
	<form onsubmit={submit}>
		<div class="card-header">
			{#if persistedOnly == null}
				:new: {db.userState.domain}
			{:else}
				<strong>{db.userState.domain}</strong>{#if dirty}*{/if}
			{/if}
			<button type="submit" class="btn {buttonClass()}" onclick={submit}>Save</button>
		</div>
		<div class="card-body">
			<label for="domain-note">Note:</label>
			<input type="text" class="form-control" id="domain-note" bind:value={db.userState.domainConfig.note} />

			<label for="domain-length">Length:</label>
			<input type="number" class="form-control" id="domain-length" bind:value={db.userState.domainConfig.length} />

			<label for="domain-suffix">Suffix:</label>
			<input type="text" class="form-control" id="domain-suffix" bind:value={db.userState.domainConfig.suffix} />
		</div>
	</form>
</div>
