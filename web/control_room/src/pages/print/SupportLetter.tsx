// The support letter for one farmer and their farms, as an official A4 page. Print it or save it as PDF
// from the browser. The letter's language can differ from the site's (a Kurdish letter from an English
// screen, for example); it follows the site language at first.
import { useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Printer, ArrowLeft } from 'lucide-react';
import { EN, KU, useI18n, type Vars } from '../../i18n';
import { db } from '../../data/db';
import { useDoc, useRows } from '../../data/store';
import { farmsOf, letterNumber, govByName, distByName, subByKey } from '../../data/api';
import type { Bi, Lang } from '../../data/types';
import { GrainSun } from '../../motion/GrainSun';
import './print.css';

/** Text, numbers and dates in the letter's own language. */
function useLetterLang(ll: Lang, kurdishDigits: boolean) {
  return useMemo(() => {
    const locale = ll === 'ku' ? (kurdishDigits ? 'ckb-IQ-u-nu-arab' : 'ckb-IQ-u-nu-latn') : 'en-GB';
    const nf = [0, 1].map(d => new Intl.NumberFormat(locale, { minimumFractionDigits: d, maximumFractionDigits: d }));
    const df = new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'long', year: 'numeric' });
    const own = ll === 'ku' ? KU : EN;
    const lt = (k: string, v?: Vars) => {
      const s = own['letter.' + k] || EN['letter.' + k] || k;
      return v ? s.replace(/\{(\w+)\}/g, (m, x) => (x in v ? String(v[x]) : m)) : s;
    };
    return {
      lt,
      n: (x: number, d: 0 | 1 = 0) => nf[d].format(x),
      d: (iso: string | Date) => df.format(new Date(iso)),
      b: (x: Bi | undefined) => (x ? (ll === 'ku' ? x.ku || x.en : x.en || x.ku) : ''),
      nm: (x: { en: string; ku?: string } | undefined) => (x ? (ll === 'ku' ? x.ku || x.en : x.en) : ''),
    };
  }, [ll, kurdishDigits]);
}

export default function SupportLetter() {
  const { id = '' } = useParams();
  const { lang } = useI18n();
  const settings = useDoc(db.settings);
  useRows(db.farms);
  useRows(db.farmers);
  const farmer = db.farmers.get(id);
  const [ll, setLl] = useState<Lang>(lang);
  const [purpose, setPurpose] = useState('');
  const L = useLetterLang(ll, settings.kurdishDigits);
  const dir = ll === 'ku' ? 'rtl' : 'ltr';

  if (!farmer) {
    return <div className="print-desk"><div className="print-page" dir={dir}><p>{L.lt('not_found')}</p><Link to="/admin/farms" className="btn">{L.lt('back')}</Link></div></div>;
  }
  const farms = farmsOf(farmer.id);
  const area = farms.reduce((a, f) => a + f.area, 0);
  const byCrop = new Map<string, number>();
  for (const f of farms) for (const c of f.crops) byCrop.set(c.crop, (byCrop.get(c.crop) ?? 0) + c.dunam);
  const no = letterNumber(settings.letterPrefix, farmer.id);
  const today = new Date();
  const sep = ll === 'ku' ? '، ' : ', ';
  const placeOf = (dist: string, sub: string, gov: string) =>
    [L.nm(subByKey.get(dist + '|' + sub)) || sub, L.nm(distByName.get(dist)) || dist, L.nm(govByName.get(gov)) || gov].filter((x, i, a) => x && a.indexOf(x) === i).join(sep);
  const cropName = (c: string) => L.b(db.crops.get(c)?.name) || c;

  return (
    <div className="print-desk">
      <div className="print-bar" dir={lang === 'ku' ? 'rtl' : 'ltr'}>
        <Link to={'/admin/farms?farmer=' + farmer.id} className="btn"><ArrowLeft className="flip-rtl" />{L.lt('back')}</Link>
        <span className="grow" />
        <span className="muted small">{L.lt('letter_lang')}</span>
        <div className="lang" role="group" aria-label={L.lt('letter_lang')}>
          <button className={'ku' + (ll === 'ku' ? ' on' : '')} onClick={() => setLl('ku')} lang="ckb">کوردی</button>
          <button className={ll === 'en' ? 'on' : ''} onClick={() => setLl('en')} lang="en">EN</button>
        </div>
        <button className="btn primary" onClick={() => window.print()}><Printer />{L.lt('print')}</button>
      </div>

      <article className="print-page" dir={dir} lang={ll === 'ku' ? 'ckb' : 'en'} style={{ fontFamily: ll === 'ku' ? 'var(--arabic)' : 'var(--latin)' }}>
        <header className="print-head">
          <div className="logo"><GrainSun size={38} /></div>
          <div className="org"><b>{L.b(settings.orgName)}</b><span>{L.lt('issuer')}</span></div>
          <div className="ref">
            <div>{L.lt('number')}: <b>{no}</b></div>
            <div>{L.lt('date')}: {L.d(today)}</div>
          </div>
        </header>

        <div className="print-title">
          <div className="eyebrow">{L.lt('kicker')}</div>
          <h1>{L.lt('title')}</h1>
        </div>

        <p><b>{L.lt('to')}</b></p>
        <p>{L.lt('body', {
          name: L.b(farmer.name), phone: farmer.phone, place: placeOf(farmer.dist, farmer.sub, farmer.gov),
          since: L.d(farmer.joined), n: L.n(farms.length), area: L.n(area, 1),
        })}</p>

        <h2>{L.lt('farmer_details')}</h2>
        <dl className="print-facts">
          <div><dt>{L.lt('name')}</dt><dd><bdi>{L.b(farmer.name)}</bdi></dd></div>
          <div><dt>{L.lt('farmer_no')}</dt><dd>{farmer.id}</dd></div>
          <div><dt>{L.lt('phone')}</dt><dd><span className="ltr">{farmer.phone}</span></dd></div>
          <div><dt>{L.lt('gender')}</dt><dd>{L.lt('g_' + farmer.gender)}</dd></div>
          <div><dt>{L.lt('place')}</dt><dd>{placeOf(farmer.dist, farmer.sub, farmer.gov)}</dd></div>
          <div><dt>{L.lt('village')}</dt><dd><bdi>{farmer.village || '-'}</bdi></dd></div>
          <div><dt>{L.lt('registered')}</dt><dd>{L.d(farmer.joined)}</dd></div>
          <div><dt>{L.lt('status')}</dt><dd>{L.lt('st_' + farmer.status)}</dd></div>
        </dl>

        <h2>{L.lt('farms_title', { n: L.n(farms.length) })}</h2>
        {farms.length === 0 ? <p>{L.lt('no_farms')}</p> : (
          <div className="print-scroll">
            <table className="pt">
              <thead><tr>
                <th>#</th><th>{L.lt('farm')}</th><th>{L.lt('place')}</th><th>{L.lt('village')}</th>
                <th className="num">{L.lt('area')}</th><th>{L.lt('crops')}</th><th>{L.lt('irrigation')}</th><th>{L.lt('registered')}</th>
              </tr></thead>
              <tbody>
                {farms.map(f => (
                  <tr key={f.id}>
                    <td>{f.id}</td><td><bdi>{f.name}</bdi></td><td>{placeOf(f.dist, f.sub, f.gov)}</td><td><bdi>{f.village || '-'}</bdi></td>
                    <td className="num">{L.n(f.area, 1)}</td>
                    <td>{f.crops.map(c => `${cropName(c.crop)} ${L.n(c.dunam, 1)}`).join(sep)}</td>
                    <td>{L.lt('irr_' + f.irrigation)}</td><td>{L.d(f.created)}</td>
                  </tr>
                ))}
                <tr className="total"><td colSpan={4}>{L.lt('total')}</td><td className="num">{L.n(area, 1)}</td><td colSpan={3}>{[...byCrop].sort((a, z) => z[1] - a[1]).map(([c, a]) => `${cropName(c)} ${L.n(a, 1)}`).join(sep)}</td></tr>
              </tbody>
            </table>
          </div>
        )}
        <p className="print-note">{L.lt('unit_note')}</p>

        <h2>{L.lt('purpose_title')}</h2>
        <div className="screen-only">
          <input className="print-input" type="text" dir="auto" value={purpose} onChange={e => setPurpose(e.target.value)} placeholder={L.lt('purpose_ph')} />
        </div>
        <p className="print-only">{purpose ? <bdi>{purpose}</bdi> : L.lt('purpose_none')}</p>

        <p>{L.lt('closing')}</p>

        <div className="print-sign">
          <div>
            <div>{L.lt('signed')}</div>
            <div className="line"><b>{L.b(settings.letterSigner)}</b><div className="print-note">{L.b(settings.letterSignerTitle)}</div></div>
          </div>
          <div className="stamp">{L.lt('stamp')}</div>
        </div>

        <footer className="print-foot">{L.lt('check', { phone: settings.helpPhone, no })}</footer>
      </article>
    </div>
  );
}
