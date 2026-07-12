<script lang="ts">
import { Db } from "./Db";
let { db } : { db: Db } = $props();
let syncState = $derived(db.syncState());
</script>

{#if syncState === 'stale'}
	<span class="fs-5 lh-1" style="position:relative; top:0.2em;">
		&#8634;
	</span>
{/if}
<button tabindex="-1" class='btn btn-outline-light ms-3' onclick={db.sync}>sync</button>
