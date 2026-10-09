// The branded intro (design frame 01). Only on the first visit, when the cache is empty: it loads the
// public data the first page needs, and its 00 to 100 follows those real calls. Later visits open from
// the cache with skeletons. Any key or a click skips it; it never waits more than 8 s.
import { useEffect, useRef, useState } from 'react';
import { Check, Loader } from 'lucide-react';
import { GrainSun } from './GrainSun';
import { useI18n } from '../i18n';
import { PRELOAD } from '../api/public';
import { isWarm, markWarm } from '../api/cache';

export const reducedMotion = () => typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;
export const introWanted = () => !isWarm() && !reducedMotion();

const STEPS = ['intro.s_zones', 'intro.s_fires', 'intro.s_dams', 'intro.s_brief'];

export function Preloader({ onDone }: { onDone: () => void }) {
  const { t, num } = useI18n();
  const [done, setDone] = useState<boolean[]>(() => STEPS.map(() => false));
  const [shown, setShown] = useState(0);
  const [lift, setLift] = useState(false);
  const finished = useRef(false);
  const target = useRef(0);
  const cb = useRef(onDone); cb.current = onDone;

  const finish = () => {
    if (finished.current) return;
    finished.current = true;
    markWarm();
    setShown(100);
    setTimeout(() => setLift(true), 220);
    setTimeout(() => { document.body.classList.remove('pre-on'); cb.current(); }, 220 + 900);
  };

  useEffect(() => {
    document.body.classList.add('pre-on');
    const t0 = performance.now();
    PRELOAD.forEach((run, i) => run().then(() => setDone(d => { const n = [...d]; n[i] = true; target.current = n.filter(Boolean).length / n.length * 100; return n; })));
    // the shown number eases towards the real progress; at least 1.4 s so the logo can draw itself
    const iv = window.setInterval(() => {
      const el = performance.now() - t0;
      setShown(s => {
        const cap = Math.min(target.current, el < 1400 ? (el / 1400) * 100 : 100);
        const next = s + (cap - s) * 0.12;
        if (target.current >= 100 && el >= 1400 && next > 99.5) { clearInterval(iv); finish(); return 100; }
        return next;
      });
      if (el > 8000) { clearInterval(iv); finish(); }
    }, 30);
    const skip = () => { clearInterval(iv); finish(); };
    addEventListener('keydown', skip, { once: true });
    return () => { clearInterval(iv); removeEventListener('keydown', skip); document.body.classList.remove('pre-on'); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const n = Math.floor(shown);
  const current = done.findIndex(d => !d);
  return (
    <div className={'pre' + (lift ? ' lift' : '')} onClick={finish} aria-hidden="true">
      <div className="pre-center">
        <GrainSun size={140} draw />
        <div className="pre-word"><span className="en">Jutyar</span><span className="ku">جوتیار</span></div>
        <div className="pre-say"><span key={current}>{current === -1 ? t('intro.ready') : t(STEPS[current] + '_doing')}</span></div>
        <div className="pre-bar"><i style={{ width: n + '%' }} /></div>
        <ul className="pre-steps">
          {STEPS.map((s, i) => <li key={s} className={done[i] ? 'ok' : ''}>{done[i] ? <Check /> : <Loader className="spin" />}<span>{t(s)}</span></li>)}
        </ul>
      </div>
      <div className="pre-count">{num(n)}<small>%</small></div>
      <div className="pre-skip">{t('intro.skip')}</div>
    </div>
  );
}
