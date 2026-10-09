// Gold cursor: a dot on the pointer and a ring that follows smoothly and grows over things you can click.
// Buttons do not move (the old magnetic pull is removed). Only on a mouse, never on touch screens.
// Cost: one passive mousemove listener; the ring's animation loop runs only while it is catching up.
import { useEffect, useRef } from 'react';
import { reducedMotion } from './Preloader';

const HOT = 'a, button, .click, .chip, label.switch, select, summary, [role=button], tr.click';
const TEXT = 'input, textarea, select, [contenteditable], .leaflet-container';

export function Cursor() {
  const dot = useRef<HTMLDivElement>(null), ring = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!matchMedia('(pointer: fine)').matches || reducedMotion()) return;
    const html = document.documentElement;
    html.classList.add('has-cursor');
    let mx = -100, my = -100, rx = -100, ry = -100, raf = 0;
    let shown = false;
    const follow = () => {
      rx += (mx - rx) * 0.2; ry += (my - ry) * 0.2;
      ring.current!.style.transform = `translate3d(${rx}px,${ry}px,0)`;
      raf = Math.abs(mx - rx) + Math.abs(my - ry) > 0.3 ? requestAnimationFrame(follow) : 0;
    };
    const move = (e: MouseEvent) => {
      mx = e.clientX; my = e.clientY;
      dot.current!.style.transform = `translate3d(${mx}px,${my}px,0)`;
      if (!shown) { shown = true; enter(); }
      if (!raf) raf = requestAnimationFrame(follow);
      // the glow under the cursor: only the card or hero under the pointer gets the two numbers
      const el = (e.target as Element).closest?.('.card, .hero') as HTMLElement | null;
      if (el) {
        const r = el.getBoundingClientRect();
        if (el.classList.contains('hero')) { el.style.setProperty('--hx', mx - r.left + 'px'); el.style.setProperty('--hy', my - r.top + 'px'); }
        else { el.style.setProperty('--mx', mx - r.left + 'px'); el.style.setProperty('--my', my - r.top + 'px'); }
      }
    };
    const over = (e: MouseEvent) => {
      const t = e.target as Element;
      const text = !!t.closest?.(TEXT);
      ring.current!.classList.toggle('text', text);
      dot.current!.classList.toggle('hidden', text);
      ring.current!.classList.toggle('big', !text && !!t.closest?.(HOT));
    };
    const leave = () => { shown = false; ring.current!.classList.add('hidden'); dot.current!.classList.add('hidden'); };
    const enter = () => { ring.current!.classList.remove('hidden'); dot.current!.classList.remove('hidden'); };
    addEventListener('mousemove', move, { passive: true });
    document.addEventListener('mouseover', over, { passive: true });
    document.documentElement.addEventListener('mouseleave', leave);
    document.documentElement.addEventListener('mouseenter', enter);
    return () => {
      cancelAnimationFrame(raf);
      html.classList.remove('has-cursor');
      removeEventListener('mousemove', move);
      document.removeEventListener('mouseover', over);
      document.documentElement.removeEventListener('mouseleave', leave);
      document.documentElement.removeEventListener('mouseenter', enter);
    };
  }, []);
  return <><div ref={dot} className="cursor-dot hidden" /><div ref={ring} className="cursor-ring hidden" /></>;
}
