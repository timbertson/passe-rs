<script lang="ts">
import {UserCtl} from './User';
const { ctl } : { ctl: UserCtl } = $props();
</script>

<form onsubmit={ctl.submitLogin}>
<div class="row">
	<div class="col">
		<span class="fs-4">passe</span>
	</div>
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="col input-group flex-nowrap">
			<input type="text" class="form-control" placeholder="user" id="user" bind:value={ctl.state.user} />
			<input type="password" class="form-control" placeholder="password" id="user-password" bind:value={ctl.state.password} />
			<div class="btn-group ms-2">
				<button type="submit" class="btn btn-primary">Login</button>
			</div>
			<div class="btn-group ms-2">
				<button type="submit" class="btn btn-secondary" onclick={ctl.submitRegister}>Register</button>
			</div>
	</div>
</div>

		</form>
