<script lang="ts">
import { Db } from "./Db";
let { db } : { db: Db } = $props();
</script>

<button class='btn btn-info' onclick={db.sync}>sync</button>
{#if db.userState.syncState === 'stale'}
	:stale:
{/if}
