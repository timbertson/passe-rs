import { mount } from 'svelte';
import App from './App.svelte';
import { notNull } from './util';

const elem = notNull(document.getElementById('app'));
elem.innerHTML = "";
try {
	mount(App, { target: elem });
} catch(e) {
	elem.innerText = String(e);
}

// export default app;
