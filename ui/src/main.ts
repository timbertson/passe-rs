import { mount } from 'svelte';
import App from './App.svelte';

mount(App, {
	target: document.getElementById('app'),
	props: {
		name: 'world'
	}
});

// export default app;
