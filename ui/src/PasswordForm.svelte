<script lang="ts">
import { Db } from "./Db.js";
import { EMPTY, notNull } from './util.js';

let password = $state(EMPTY);

let maskPassword = $state(true);
let generatedPassword = $state(EMPTY);

const { db }: { db: Db } = $props();

let domain = () => db.userState.domain;

let showSuggestions = $state(false);

let domainSuggestions = $derived.by(() => {
	const domain = db.userState.domain;
	if (domain.length < 2) {
		return [];
	}
	return db.domainSuggestions(domain);
});

let selectedSuggestion: number|null = $derived.by(() => {
	const _ = domainSuggestions;
	return null
});

function mask(password: String) {
	if (maskPassword) {
		return '●'.repeat(password.length);
	} else {
		return password;
	}
}

function generatedPasswordInput(): null|HTMLInputElement {
	return document.querySelector('.password-display input.password');
}

function generatedPasswordDummy(): null|HTMLInputElement {
	return document.querySelector('.password-display .dummy');
}

async function copyToClipboard(data: string): Promise<boolean> {
	try {
		const clipboard = navigator.clipboard;
		if (!clipboard) {
			db.setToast('Clipboard API unavailable');
			return false;
		}
		await clipboard.writeText(data);
		db.setToast('Copied to clipboard!');
		console.info("Copied to clipboard");
		return true;
	} catch(e) {
		db.setToast(`Can't copy to clipboard: ${e}`);
		console.error("Error copying clipboard:", e);
		return false
	}
}

async function generate(ev: Event) {
	ev.preventDefault();
	if (domain() == EMPTY || password == EMPTY) {
		console.info('empty domain or password');
	} else {
		generatedPassword = db.generatePassword(domain(), password);
		await copyToClipboard(generatedPassword);
		setTimeout(() => generatedPasswordInput()?.focus(),5);
	}
}

function baseKeydown(ev: KeyboardEvent) {
	const code = ev.code;
	// console.log("KEY: ", code);
	if (code == 'Escape') {
		ev.preventDefault();
		const id = (ev.target as Element).getAttribute('id');
		if (id === 'domain') {
			db.userState.domain = EMPTY;
		}

		console.info('clearing password');
		clearPassword()
	}
}

function domainKeydown(ev: KeyboardEvent) {
	const code = ev.code;
	function acceptActiveSuggestion() {
		if (selectedSuggestion != null) {
			db.userState.domain = domainSuggestions[selectedSuggestion];
		}
	}

	if (code == 'ArrowDown') {
		ev.preventDefault();
		nextSelectedSuggestion(1);
	} else if (code == 'ArrowUp') {
		ev.preventDefault();
		nextSelectedSuggestion(-1);
	} else if (code == 'Tab') {
		acceptActiveSuggestion();
	} else if (code == 'Enter') {
		if (db.userState.password == '') {
			ev.preventDefault(); // don't submit; tab instead
			acceptActiveSuggestion();
			notNull(document.getElementById('domain-password')).focus();
		}
	}
}

function nextSelectedSuggestion(diff: number) {
	if (selectedSuggestion == null) {
		console.log("null; set to 0");
		selectedSuggestion = 0;
	} else {
		console.log(`non-null; adding ${diff} to ${selectedSuggestion}`);
		selectedSuggestion = Math.min(
			domainSuggestions.length - 1,
			Math.max(0, selectedSuggestion + diff)
		);
	}
}

export function clearGenerated() {
	generatedPassword = EMPTY;
	maskPassword = true;
}

function clearPassword() {
	generatedPassword = EMPTY;
	password = EMPTY;
}

function setDomain(value: string) {
	return function(ev: Event) {
		ev.preventDefault();
		db.userState.domain = value;
	}
}

function setSelectedSuggestion(idx: number | null) {
	return function(ev: Event) {
		ev.preventDefault();
		selectedSuggestion = idx;
	}
}

function suggestionClass(idx: number) {
	return idx === selectedSuggestion ? 'active' : '';
}

function setShowSuggestions(value: boolean) {
	return function(ev: Event) {
		ev.preventDefault();
		selectedSuggestion = null;
		showSuggestions = value;
	}
}

function generatedFocus(ev: Event) {
	generatedPasswordInput()?.select();
	generatedPasswordDummy()?.classList?.add('selected');
}

function generatedBlur(ev: Event) {
	generatedPasswordDummy()?.classList.remove('selected');
}

function toggleMask(ev: Event) {
	ev.preventDefault();
	maskPassword = !maskPassword;
}

function stopPropagation(ev: Event) {
	ev.stopPropagation();
}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<form onsubmit={generate} onkeydown={baseKeydown} class="password-form" autocapitalize="none">
	<div class="mb-3">
		<label for="domain" class="form-label">Domain</label>
		<!-- svelte-ignore a11y_autofocus -->
		<input type="text"
			class="form-control"
			id="domain"
			bind:value={db.userState.domain}
			onkeydown={domainKeydown}
			onfocus={setShowSuggestions(true)}
			onblur={setShowSuggestions(false)}
			autofocus
		/>

		{#if showSuggestions && domainSuggestions.length > 0}
			<ul class="dropdown-menu show" onmouseleave={setSelectedSuggestion(null)}>
				{#each domainSuggestions as suggestion, i}
					<li><a
						class="dropdown-item {suggestionClass(i)}"
						href="#null"
						tabindex="-1"
						onmousedown={setDomain(suggestion)}
						onmouseenter={setSelectedSuggestion(i)}
					>{suggestion}</a></li>
				{/each}
			</ul>
		{/if}

	</div>


	<div class="mb-3">
		<label for="domain-password" class="form-label">Password</label>

		<div class="input-group">
			<input type="password" class="form-control" id="domain-password" bind:value={password} onkeydown={baseKeydown} oninput={clearGenerated} />
			<button type="submit" class="btn btn-secondary">Submit</button>
		</div>

		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		{#if generatedPassword !== EMPTY}
			<div class="overlay" onclick={clearPassword}></div>
			<div class="popover show password-display" role="alert" data-popper-placement="right">
				<div class="popover-body">
					<div class="row">
						<div class="col">
							<!-- Generated password: -->
							<div class="password dummy">{mask(generatedPassword)}
								<input type="text" name="generated-password" class="password password-value" onfocus={generatedFocus} onblur={generatedBlur} value="{generatedPassword}"/>
							</div>
						</div>
						<div class="col text-end">
							<button class="btn btn-primary" onclick={toggleMask}>show</button>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- <button type="button" class="btn btn-secondary" onclick={clearGenerated}>Clear</button> -->
</form>
