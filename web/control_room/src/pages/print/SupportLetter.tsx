// Support letter (design lcz38). A preview is drawn from the farmer and their farms; the letter is only
// issued (POST /dashboard/farmers/{id}/letters) when staff press "Issue letter" and confirm, because every
// call stores a new number (FRONTEND.md 9). Then the printable A4 uses exactly what the server returned.
// ?number=JTY-... checks a letter issued earlier.
import { useMemo, useState } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';
import { Printer, ArrowRight, Stamp, RotateCcw, Search } from 'lucide-react';
import { useI18n, EN, KU } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs, ApiError } from '../../api/client';
import { useApi } from '../../api/cache';
import { Confirm, Note, useToast } from '../../components/ui';
import { StateBox, useErrorText } from '../../components/domain';
import { GrainSun } from '../../motion/GrainSun';
import { CROP } from '../../data/crops';
import { DISTRICT_BY_SLUG, SUB_BY_SLUG, GOV_BY_NAME } from '../../data/places';
import type { Farmer, FarmSummary, IssuedLetter, LetterRecord } from '../admin/farms/common';
import './print.css';

type L = 'ku' | 'en';
const KU_MONTHS = ['کانوونی دووەم', 'شوبات', 'ئازار', 'نیسان', 'ئایار', 'حوزەیران', 'تەممووز', 'ئاب', 'ئەیلوول', 'تشرینی یەکەم', 'تشرینی دووەم', 'کانوونی یەکەم'];

export default function SupportLetter() {
  const { farmerId = '' } = useParams();
  const [params, setParams] = useSearchParams();
  const { t: siteT, lang: siteLang, date } = useI18n();
  const { can } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const [lang, setLang] = useState<L>(siteLang);
  const [purpose, setPurpose] = useState('');
  const [letter, setLetter] = useState<IssuedLetter | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [busy, setBusy] = useState(false);
  const [checkNo, setCheckNo] = useState(params.get('number') ?? '');
  const number = params.get('number');

  const farmerQ = useApi<{ farmer: Farmer }>(farmerId ? '/dashboard/farmers/' + farmerId : null, ['farmers'], { auth: true });
  const farmer = farmerQ.data?.farmer;
  const farmsQ = useApi<{ farms: FarmSummary[] }>(farmer && can('farms') ? '/dashboard/farms' + qs({ owner_phone: farmer.phone, rows_per_page: 100 }) : null, ['farms'], { auth: true });
  const record = useApi<{ letter: LetterRecord }>(number ? '/dashboard/letters/' + encodeURIComponent(number) : null, [], { auth: true });

  // the letter's own language, independent of the site language
  const lt = (key: string, vars?: Record<string, string | number>) => {
    const s = (lang === 'ku' ? KU['letter.' + key] : undefined) || EN['letter.' + key] || key;
    return vars ? s.replace(/\{(\w+)\}/g, (m, k) => (k in vars ? String(vars[k]) : m)) : s;
  };
  const nf = useMemo(() => new Intl.NumberFormat(lang === 'ku' ? 'ckb-IQ-u-nu-latn' : 'en-GB', { maximumFractionDigits: 1, minimumFractionDigits: 1 }), [lang]);
  // Kurdish month names written out: browsers without ckb data would fall back to English
  const fmtDate = (iso: string) => { const d = new Date(iso); return lang === 'ku' ? `${d.getDate()}ی ${KU_MONTHS[d.getMonth()]}ی ${d.getFullYear()}` : d.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' }); };
  const n1 = (v: number) => nf.format(v);
  const nm = (p?: { en: string; ku: string }) => (p ? (lang === 'ku' ? p.ku || p.en : p.en) : '');
  const placeOf = (x: { governorate?: string | null; zone_slug?: string | null; sub_zone_slug?: string | null; village?: string | null }) =>
    [x.village, nm(x.sub_zone_slug ? SUB_BY_SLUG.get(x.sub_zone_slug) : undefined), nm(x.zone_slug ? DISTRICT_BY_SLUG.get(x.zone_slug) : undefined), nm(x.governorate ? GOV_BY_NAME.get(x.governorate) : undefined)].filter(Boolean).join(lang === 'ku' ? '، ' : ', ');
  const cropText = (cs: { crop: string; dunam: number }[]) => cs.filter(c => c.crop !== 'empty').map(c => `${lang === 'ku' ? CROP.get(c.crop)?.ku ?? c.crop : CROP.get(c.crop)?.en ?? c.crop} ${n1(c.dunam)}`).join(' · ') || '-';

  // what the page shows: the issued letter, or a preview from the current data
  const view = useMemo(() => {
    if (letter) return { number: letter.number, created: letter.created_at, issuer: letter.issued_by.name, farmer: letter.farmer, farms: letter.farms, totals: letter.totals, purpose: letter.purpose };
    if (!farmer) return null;
    const farms = farmsQ.data?.farms ?? [];
    const totalsMap = new Map<string, number>();
    for (const f of farms) for (const c of f.crops) totalsMap.set(c.crop, (totalsMap.get(c.crop) ?? 0) + c.dunam);
    return {
      number: null as string | null, created: new Date().toISOString(), issuer: '', farmer, purpose: purpose.trim(),
      farms: farms.map(f => ({ id: f.id, name: f.name, area_dunam: f.area_dunam, crops: f.crops, governorate: f.governorate, zone_slug: f.zone_slug, sub_zone_slug: f.sub_zone_slug })),
      totals: { farms: farms.length, dunam: farms.reduce((a, f) => a + f.area_dunam, 0), crops: [...totalsMap].map(([crop, dunam]) => ({ crop, dunam })) },
    };
  }, [letter, farmer, farmsQ.data, purpose]);

  const issue = async () => {
    if (busy || !farmer) return;
    setBusy(true);
    try {
      const r = await api.post<{ letter: IssuedLetter }>(`/dashboard/farmers/${farmer.id}/letters`, { purpose: purpose.trim(), lang });
      setLetter(r.letter);
      toast(siteT('letter.issued', { no: r.letter.number }), 'good');
    } catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setBusy(false); }
  };

  const purposeOk = purpose.trim().length >= 3 && purpose.trim().length <= 300;
  const back = '#/admin/farms?farmer=' + farmerId;

  if (number) {
    const r = record.data?.letter;
    return (
      <div className="print-desk">
        <div className="print-bar"><a className="btn" href={back}><ArrowRight className="flip-rtl" />{siteT('letter.back')}</a><span className="grow" /></div>
        <div className="print-page" style={{ minHeight: 0 }}>
          <h1>{siteT('letter.check_title')}</h1>
          {record.loading && <div className="sk-rows">{[0, 1, 2].map(i => <i key={i} className="sk" />)}</div>}
          {record.error && <StateBox kind={record.error.status === 404 ? 'empty' : 'error'} title={record.error.status === 404 ? siteT('letter.check_none', { no: number }) : undefined} />}
          {r && <>
            <Note tone="good">{siteT('letter.check_ok')}</Note>
            <dl className="facts" style={{ marginTop: 12 }}>
              <dt>{siteT('letter.number')}</dt><dd className="ltr">{r.number}</dd>
              <dt>{siteT('letter.name')}</dt><dd>{r.farmer_name || '-'} <span className="muted">#{r.farmer_id}</span></dd>
              <dt>{siteT('letter.date')}</dt><dd>{date(r.created_at)}</dd>
              <dt>{siteT('letter.purpose_title')}</dt><dd><bdi>{r.purpose}</bdi></dd>
              <dt>{siteT('letter.letter_lang')}</dt><dd>{r.lang === 'ku' ? 'کوردی' : 'English'}</dd>
            </dl>
          </>}
        </div>
      </div>
    );
  }

  if (farmerQ.error) return <div className="print-desk"><div className="print-page" style={{ minHeight: 0 }}><StateBox kind={farmerQ.error.status === 404 ? 'empty' : 'error'} title={farmerQ.error.status === 404 ? siteT('letter.not_found') : undefined} /></div></div>;

  return (
    <div className="print-desk" dir={lang === 'ku' ? 'rtl' : 'ltr'}>
      <div className="print-bar" dir={siteLang === 'ku' ? 'rtl' : 'ltr'}>
        <a className="btn" href={back}><ArrowRight className="flip-rtl" />{siteT('letter.back')}</a>
        <div className="lang" role="group" aria-label={siteT('letter.letter_lang')}>
          <button className={'ku' + (lang === 'ku' ? ' on' : '')} disabled={!!letter} onClick={() => setLang('ku')}>کوردی</button>
          <button className={lang === 'en' ? 'on' : ''} disabled={!!letter} onClick={() => setLang('en')}>EN</button>
        </div>
        <span className="grow" />
        {!letter && can('farmers', 'create') && <button className="btn primary" disabled={busy || !purposeOk || !farmer} onClick={() => setConfirm(true)}><Stamp />{busy ? siteT('common.loading') : siteT('letter.issue')}</button>}
        {letter && <button className="btn" onClick={() => { setLetter(null); setPurpose(''); }}><RotateCcw />{siteT('letter.issue_another')}</button>}
        <button className="btn primary" disabled={!letter} onClick={() => window.print()} title={letter ? undefined : siteT('letter.print_after')}><Printer />{siteT('letter.print')}</button>
      </div>
      <div className="print-bar screen-only" dir={siteLang === 'ku' ? 'rtl' : 'ltr'} style={{ paddingTop: 0 }}>
        <form className="row" style={{ gap: 6 }} onSubmit={e => { e.preventDefault(); if (checkNo.trim()) setParams({ number: checkNo.trim() }); }}>
          <input type="text" value={checkNo} onChange={e => setCheckNo(e.target.value)} placeholder="JTY-202610-1-1" className="ltr-input" style={{ width: 200 }} aria-label={siteT('letter.check_title')} />
          <button className="btn sm" type="submit"><Search />{siteT('letter.check_btn')}</button>
        </form>
        {!letter && <Note tone="info">{siteT('letter.preview_note')}</Note>}
      </div>
      {!view ? <div className="print-page"><div className="sk-rows">{[0, 1, 2, 3, 4, 5].map(i => <i key={i} className="sk" />)}</div></div> : (
        <div className="print-page">
          <div className="print-head">
            <div className="logo"><GrainSun size={34} /></div>
            <div className="org"><b>{lt('org')}</b><span>{lt('issuer')}</span></div>
            <div className="ref">
              <div>{lt('number')}: {view.number ? <b className="ltr">{view.number}</b> : <span className="muted">{lt('number_pending')}</span>}</div>
              <div>{lt('date')}: {fmtDate(view.created)}</div>
            </div>
          </div>
          <div className="print-title"><div className="eyebrow">{lt('kicker')}</div><h1>{lt('title')}</h1></div>
          <p><b>{lt('to')}</b></p>
          <p>{lt('body', { name: '⁨' + (view.farmer.name || '-') + '⁩', phone: '⁦' + view.farmer.phone + '⁩', place: placeOf(view.farmer) || '-', n: view.totals.farms, area: n1(view.totals.dunam) })}</p>
          <h2>{lt('farmer_details')}</h2>
          <dl className="print-facts">
            <div><dt>{lt('name')}</dt><dd><bdi>{view.farmer.name || '-'}</bdi></dd></div>
            <div><dt>{lt('farmer_no')}</dt><dd>{view.farmer.id}</dd></div>
            <div><dt>{lt('phone')}</dt><dd className="ltr">{view.farmer.phone}</dd></div>
            <div><dt>{lt('gender')}</dt><dd>{view.farmer.gender ? lt('g_' + view.farmer.gender) : '-'}</dd></div>
            <div><dt>{lt('place')}</dt><dd>{placeOf({ ...view.farmer, village: null }) || '-'}</dd></div>
            <div><dt>{lt('village')}</dt><dd><bdi>{view.farmer.village || '-'}</bdi></dd></div>
          </dl>
          <h2>{lt('farms_title', { n: view.totals.farms })}</h2>
          {view.farms.length ? (
            <div className="print-scroll"><table className="pt">
              <thead><tr><th>#</th><th>{lt('farm')}</th><th>{lt('place')}</th><th className="num">{lt('area')}</th><th>{lt('crops')}</th></tr></thead>
              <tbody>
                {view.farms.map(f => <tr key={f.id}><td>{f.id}</td><td className="nowrap"><bdi>{f.name}</bdi></td><td>{placeOf(f) || '-'}</td><td className="num">{n1(f.area_dunam)}</td><td>{cropText(f.crops)}</td></tr>)}
                <tr className="total"><td colSpan={3}>{lt('total')}</td><td className="num">{n1(view.totals.dunam)}</td><td>{cropText(view.totals.crops)}</td></tr>
              </tbody>
            </table></div>
          ) : <p className="print-note">{lt('no_farms')}</p>}
          <p className="print-note">{lt('unit_note')}</p>
          <h2>{lt('purpose_title')}</h2>
          {letter ? <p><bdi>{view.purpose}</bdi></p> : (
            <div className="screen-only">
              <input className="print-input" type="text" value={purpose} maxLength={300} onChange={e => setPurpose(e.target.value)} placeholder={lt('purpose_ph')} />
              {!purposeOk && purpose.length > 0 && <div className="small" style={{ color: 'var(--danger)' }}>{siteT('letter.purpose_len')}</div>}
            </div>
          )}
          <p>{lt('closing')}</p>
          <div className="print-sign">
            <div><div className="muted small">{lt('signed')}</div><div className="line"><b>{view.issuer || ' '}</b><div className="print-note">{lt('signer_title')}</div></div></div>
            <div className="stamp">{lt('stamp')}</div>
          </div>
          <div className="print-foot">{view.number ? lt('check', { no: view.number }) : lt('check_pending')}</div>
        </div>
      )}
      {confirm && <Confirm danger={false} title={siteT('letter.issue_q')} text={siteT('letter.issue_text', { name: farmer?.name || farmer?.phone || '' })} okLabel={siteT('letter.issue')} onOk={issue} onClose={() => setConfirm(false)} />}
    </div>
  );
}
