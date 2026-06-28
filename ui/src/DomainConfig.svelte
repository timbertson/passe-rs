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

const buttonClass = () => canSave() ? "btn-secondary" : "disabled btn-secondary";

const headerClass = () => {
	if (persistedOnly == null) {
		return 'new';
	} else {
		return dirty ? 'dirty' : 'saved';
	}
}

</script>

<div class="card domain-config">
	<form onsubmit={submit}>
		<div class="card-header {headerClass()}">
			<div class="row">
				<div class="col fs-4">
					{#if persistedOnly == null}
						{db.userState.domain}
					{:else}
						<strong>{db.userState.domain}</strong>{#if dirty}*{/if}
					{/if}
				</div>
				<div class="col text-end">
					<button type="submit" class="btn {buttonClass()}" onclick={submit}>Save</button>
				</div>
			</div>
		</div>

		<div class="card-body">
			<div class="row">
				<div class="col">
					<label for="domain-note">Note:</label>
					<input type="text" class="form-control" id="domain-note" bind:value={db.userState.domainConfig.note} />
				</div>
			</div>
			<div class="row mt-3">
				<div class="col">
					<label for="domain-length">Length:</label>
					<input type="number" class="form-control" id="domain-length" bind:value={db.userState.domainConfig.length} />
				</div>
			</div>
			<div class="row mt-3">
				<div class="col">
					<label for="domain-suffix">Suffix:</label>
					<input type="text" class="form-control" id="domain-suffix" bind:value={db.userState.domainConfig.suffix} />
				</div>
			</div>
		</div>
	</form>
</div>
