<script lang="ts">
import { Db } from './Db';
const { db } : { db: Db } = $props();
</script>

<form onsubmit={db.submitLogin}>
	<div class="row">
		<div class="col-md">
			<span class="fs-4 mb-2">passe</span>
		</div>
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div class="col-md flex-nowrap">
			<div class="input-group">
				<input type="text" class="form-control" placeholder="user" id="user" bind:value={db.userState.user} />
				<input type="password" class="form-control" placeholder="password" id="user-password" bind:value={db.userState.password} />
				<button type="submit" class="btn btn-primary">Login</button>

				<div class="btn-group" role="group" aria-label="Default button group">
					<button type="submit" class="btn btn-link" onclick={db.submitRegister}>Register</button>
				</div>
			</div>
		</div>
	</div>
</form>
