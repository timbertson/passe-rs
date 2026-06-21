// rollup.config.js
import svelte from 'rollup-plugin-svelte';
import resolve from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';
import typescript from '@rollup/plugin-typescript';

const isRelease = (Deno.env.get('RELEASE') || 'false') == 'true';

const plugins = [
    svelte({}),
    typescript(),
    resolve({
      browser: true,
      exportConditions: ['svelte'],
      extensions: ['.svelte']
    }),
  ];

if (isRelease) {
  plugins.push(terser());
}

export default {
  onwarn(warning, dfl) {
    if (warning.code == 'CIRCULAR_DEPENDENCY') {
      // ignore completely
      // dfl(warning)
    } else {
      throw new Error(`[${warning.code}]: ${warning.message}`);
    }
  },
  input: 'src/main.ts',
  output: [
    {
      externalLiveBindings: false,
      interop: "esModule",
      format: 'es',
    },
  ],
  plugins
}
