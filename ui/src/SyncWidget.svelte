<script lang="ts">
import { Db } from "./Db";
let { db } : { db: Db } = $props();
let syncState = $derived(db.syncState());
</script>

<button class='btn btn-info' onclick={db.sync}>sync</button>
{#if syncState === 'stale'}
	:stale:
{/if}
