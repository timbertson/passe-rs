<script lang="ts">
import { Db } from "./Db";
let { db } : { db: Db } = $props();
let syncState = $derived(db.syncState());
let syncClass = $derived(syncState == 'stale' ? 'btn-light' : 'btn-outline-light');
</script>

<button tabindex="-1" class='btn {syncClass} ms-3' onclick={db.sync}>sync</button>
