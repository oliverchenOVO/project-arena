import js from '@eslint/js';
import ts from 'typescript-eslint';
export default ts.config(js.configs.recommended,...ts.configs.recommended,{
  files:['src/**/*.{ts,tsx}'],
  languageOptions:{globals:{window:'readonly',document:'readonly',console:'readonly',requestAnimationFrame:'readonly',cancelAnimationFrame:'readonly',performance:'readonly',ResizeObserver:'readonly',HTMLCanvasElement:'readonly',File:'readonly',Blob:'readonly',URL:'readonly',fetch:'readonly'}}
});
