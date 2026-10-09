// The intro: deep green curtain, the logo drawing itself, a 00 to 100 counter. At 100 the curtain lifts
// straight onto the site (the grid of squares that used to flip open is gone). Shown once per browser
// session; any key or a click skips it. Never waits more than 4 s.
import { useEffect, useRef, useState } from 'react';
import { GrainSun } from './GrainSun';
import { useI18n } from '../i18n';

const SEEN = 'jutyar.intro.seen';
export const reducedMotion = () => typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;
export function introWanted() {
  if (reducedMotion()) return false;
  try { return !sessionStorage.getItem(SEEN); } catch { return true; }
}

export function Preloader({ onDone }: { onDone: () => void }) {
  const { t } = useI18n();
  const words = [t('intro.w1'), t('intro.w2'), t('intro.w3'), t('intro.w4'), t('intro.w5'), t('intro.w6')];
  const [p, setP] = useState(0);
  const [word, setWord] = useState(0);
  const [lift, setLift] = useState(false);
  const skip = useRef(false);
  const done = useRef(onDone); done.current = onDone;

  useEffect(() => {
    document.body.classList.add('pre-on');
    let fontsOk = false;
    document.fonts?.ready.then(() => { fontsOk = true; });
    const t0 = performance.now();
    let cur = 0, last = t0, finished = false;
    const onSkip = () => { skip.current = true; };
    window.addEventListener('keydown', onSkip);
    const wi = window.setInterval(() => setWord(w => w + 1), 520);
    // a timer, not requestAnimationFrame, so the intro still ends in a background tab
    const iv = window.setInterval(() => {
      const now = performance.now(), late = now - t0 > 4000;
      const k = Math.min(1, (now - t0) / 2600);
      let target = 100 * (1 - Math.pow(1 - k, 2.2));
      if (!fontsOk && !late) target = Math.min(target, 80);
      if (late || skip.current) target = 100;
      const dt = now - last; last = now;
      cur += (target - cur) * (1 - Math.exp(-dt / (skip.current ? 40 : 140)));
      if (target - cur < 0.4) cur = target;
      setP(cur);
      if (cur >= 100 && !finished) {
        finished = true;
        clearInterval(iv); clearInterval(wi);
        window.setTimeout(() => setLift(true), 240);
        window.setTimeout(() => {
          try { sessionStorage.setItem(SEEN, '1'); } catch { /* private mode */ }
          document.body.classList.remove('pre-on');
          done.current();
        }, 240 + 900);
      }
    }, 16);
    return () => { clearInterval(iv); clearInterval(wi); window.removeEventListener('keydown', onSkip); document.body.classList.remove('pre-on'); };
  }, []);

  const n = Math.floor(p);
  const marquee = 'JUTYAR · جوتیار · FIELDS · WATER · WHEAT · BARLEY · DAMS · WEATHER · ';
  return (
    <div className={'pre' + (lift ? ' lift' : '')} onClick={() => { skip.current = true; }} aria-hidden="true">
      <div className="pre-center">
        <GrainSun size={148} draw />
        <div className="pre-word"><span className="en">Jutyar</span><span className="ku">جوتیار</span></div>
        <div className="pre-say"><span key={n >= 100 ? 'r' : word}>{n >= 100 ? t('intro.ready') : words[word % words.length]}</span></div>
      </div>
      <div className="pre-count" style={{ transform: `scale(${1 + Math.pow(p / 100, 3) * 0.5})` }}>{String(n).padStart(2, '0')}<small>%</small></div>
      <div className="pre-skip">{t('intro.skip')}</div>
      <div className="pre-marquee"><div className="mq-track"><span>{marquee.repeat(4)}</span><span>{marquee.repeat(4)}</span></div></div>
    </div>
  );
}
