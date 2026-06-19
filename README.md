# Web setup

There are a few components that feed into the web service:

 - web: rust code compiled to WASM
 - ui: Typescript svelte-based UI, using the `web` WASM module
   - built using `rollup` into a JS bundle
 - server: backend API, serves up WASM and ui bundle as static files
   - route /ui mounts ../ui/public
   - route /web mounts ../web/pkg

Building:

 - set RELEASE=true for release mode (wasm)
 - `gup server/all` for everything
 - `gup ui/all` for just client-side resources (wasm & TS)
 - `gup web/all` for just WASM changes
 - `gup ui/public/bundle.js` for just TS changes
