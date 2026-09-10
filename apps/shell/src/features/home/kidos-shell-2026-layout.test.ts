import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('./kidos-shell-2026.css', import.meta.url), 'utf8');

describe('KidOS 2026 desktop layout contract', () => {
  it('keeps the standard desktop home at four columns for 1920x1080-class viewports', () => {
    expect(css).toContain('.kidos-home-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr))');
  });

  it('keeps the compact-width fallback at two columns below 1100px', () => {
    expect(css).toContain('@media(max-width:1100px){.kidos-home-grid{grid-template-columns:repeat(2,minmax(0,1fr))}');
  });

  it('keeps the 1366x768-class height compression without changing desktop column count', () => {
    expect(css).toContain('@media(max-height:780px) and (min-width:1101px)');
    expect(css).toContain('.kidos-tile{min-height:118px}');
  });

  it('retains visible keyboard focus treatment', () => {
    expect(css).toContain(':focus-visible');
    expect(css).toContain('box-shadow:var(--kidos-focus)');
  });
});
